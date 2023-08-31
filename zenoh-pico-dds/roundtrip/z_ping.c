#include "RoundTrip.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>
#include <inttypes.h>
#include <time.h>
#include <sys/time.h>
#include <zenoh-pico.h>

/* CycloneDDS CDR Deserializer */
#include <dds/cdr/dds_cdrstream.h>

/* Some types/operations defined by CycloneDDS headers but not present in libcdr */
typedef int64_t dds_time_t;

inline dds_time_t dds_time(void)
{
  struct timespec ts;
  (void)clock_gettime(CLOCK_REALTIME, &ts);
  return (ts.tv_sec * DDS_NSECS_IN_SEC) + ts.tv_nsec;
}

const struct dds_cdrstream_allocator dds_cdrstream_default_allocator = {malloc, realloc, free};

/* The Zenoh key expr that zenoh-bridge-dds will map to DDS Topics (format: <partition>/<topic>) */
#define PING_KEY_EXPR "ping/RoundTrip"
#define PONG_KEY_EXPR "pong/RoundTrip"

#define WARMUP_TIME_MS 5000
#define TIME_STATS_SIZE_INCREMENT 50000
#define US_IN_ONE_SEC 1000000LL

#define NTP64_FRAC_MASK 0x00000000ffffffffLL /* bitmask for the fraction of a second part in a NTP64 time */
#define NANO_PER_SEC 1000000000LL            /* number of nanoseconds in 1 second */
#define FRAC_PER_SEC (1LL << 32)             /* number of NTP fraction per second (2^32) */

dds_time_t z_time_to_dds_time(uint64_t t)
{
  /* z_time_t is a NTP64 time:
      - 1st 32bits part is number of seconds since EPOCH
      - 2nd 32bits part is a fraction of second
  */
  dds_time_t sec = (t >> 32) * NANO_PER_SEC;
  uint64_t frac = t & NTP64_FRAC_MASK;
  uint64_t ns = ((frac * NANO_PER_SEC) / FRAC_PER_SEC);
  return sec + ns;
}

typedef struct ExampleTimeStats
{
  dds_time_t *values;
  unsigned long valuesSize;
  unsigned long valuesMax;
  double average;
  dds_time_t min;
  dds_time_t max;
  unsigned long count;
} ExampleTimeStats;

static void exampleInitTimeStats(ExampleTimeStats *stats)
{
  stats->values = (dds_time_t *)malloc(TIME_STATS_SIZE_INCREMENT * sizeof(dds_time_t));
  stats->valuesSize = 0;
  stats->valuesMax = TIME_STATS_SIZE_INCREMENT;
  stats->average = 0;
  stats->min = 0;
  stats->max = 0;
  stats->count = 0;
}

static void exampleResetTimeStats(ExampleTimeStats *stats)
{
  memset(stats->values, 0, stats->valuesMax * sizeof(dds_time_t));
  stats->valuesSize = 0;
  stats->average = 0;
  stats->min = 0;
  stats->max = 0;
  stats->count = 0;
}

static void exampleDeleteTimeStats(ExampleTimeStats *stats)
{
  free(stats->values);
}

static ExampleTimeStats *exampleAddTimingToTimeStats(ExampleTimeStats *stats, dds_time_t timing)
{
  if (stats->valuesSize > stats->valuesMax)
  {
    dds_time_t *temp = (dds_time_t *)realloc(stats->values, (stats->valuesMax + TIME_STATS_SIZE_INCREMENT) * sizeof(dds_time_t));
    stats->values = temp;
    stats->valuesMax += TIME_STATS_SIZE_INCREMENT;
  }
  if (stats->values != NULL && stats->valuesSize < stats->valuesMax)
  {
    stats->values[stats->valuesSize++] = timing;
  }
  stats->average = ((double)stats->count * stats->average + (double)timing) / (double)(stats->count + 1);
  stats->min = (stats->count == 0 || timing < stats->min) ? timing : stats->min;
  stats->max = (stats->count == 0 || timing > stats->max) ? timing : stats->max;
  stats->count++;

  return stats;
}

static int exampleCompareul(const void *a, const void *b)
{
  dds_time_t ul_a = *((dds_time_t *)a);
  dds_time_t ul_b = *((dds_time_t *)b);

  if (ul_a < ul_b)
    return -1;
  if (ul_a > ul_b)
    return 1;
  return 0;
}

static double exampleGetMedianFromTimeStats(ExampleTimeStats *stats)
{
  double median = 0.0;

  qsort(stats->values, stats->valuesSize, sizeof(dds_time_t), exampleCompareul);

  if (stats->valuesSize % 2 == 0)
  {
    median = (double)(stats->values[stats->valuesSize / 2 - 1] + stats->values[stats->valuesSize / 2]) / 2;
  }
  else
  {
    median = (double)stats->values[stats->valuesSize / 2];
  }

  return median;
}

static dds_time_t exampleGet99PercentileFromTimeStats(ExampleTimeStats *stats)
{
  qsort(stats->values, stats->valuesSize, sizeof(dds_time_t), exampleCompareul);
  return stats->values[stats->valuesSize - stats->valuesSize / 100];
}

static z_owned_session_t session;
static z_owned_publisher_t pub;
static z_owned_subscriber_t sub;

static ExampleTimeStats roundTrip;
static ExampleTimeStats writeAccess;
static ExampleTimeStats readAccess;
static ExampleTimeStats roundTripOverall;
static ExampleTimeStats writeAccessOverall;
static ExampleTimeStats readAccessOverall;

static dds_ostream_t os;
static RoundTripModule_DataType pub_data;

static dds_time_t startTime;
static dds_time_t preWriteTime;
static dds_time_t postWriteTime;
static dds_time_t preDeserTime;
static dds_time_t postDeserTime;
static dds_time_t elapsed = 0;

static bool warmUp = true;

// CDR Xtypes header {0x00, 0x01} indicates it's Little Endian (CDR_LE representation)
const uint8_t cdr_header[4] = {0x00, 0x01, 0x00, 0x00};
const size_t cdr_header_size = sizeof(cdr_header);

void send_ping()
{
  preWriteTime = dds_time();

  /* Serialize data before to publish:
     the serialized buffer will contain 4 bytes for the CDR header, 4 bytes for the sequence length
     and remaining will be the octets sequence
   */
  size_t alloc_size = cdr_header_size + 4 + pub_data.payload._length;
  uint8_t *buf = malloc(alloc_size);
  // Add CDR header
  memcpy(buf, cdr_header, cdr_header_size);
  os.m_buffer = buf;
  os.m_index = cdr_header_size; // Offset for CDR header
  os.m_size = alloc_size - cdr_header_size;
  os.m_xcdr_version = DDSI_RTPS_CDR_ENC_VERSION_2;

  bool ret = dds_stream_write(&os, &dds_cdrstream_default_allocator,
                              (void *)&pub_data, RoundTripModule_DataType_desc.m_ops);
  if (ret == true)
  {
    z_publisher_put_options_t options = z_publisher_put_options_default();
    z_publisher_put(z_publisher_loan(&pub), (const uint8_t *)buf, os.m_index, &options);
  }
  else
  {
    printf("ERROR serializing data\n");
    fflush(stdout);
  }
  postWriteTime = dds_time();
}

void data_handler(const z_sample_t *sample, void *arg)
{
  (void)(arg);
  dds_time_t difference = 0;

  /* Deserialize sample (no equivalent to DDS take in Zenoh, so we just measure deserialization time) */
  preDeserTime = dds_time();

  /* Allocate DataType and its octets sequence according to the received sample's payload:
     the payload contains 4 bytes for the CDR header, 4 bytes for the sequence length
     and remaining is the octets sequence
   */
  RoundTripModule_DataType data;
  data.payload._length = sample->payload.len - cdr_header_size - 4;
  data.payload._buffer = malloc(data.payload._length);
  data.payload._release = true;
  data.payload._maximum = 0;

  dds_istream_t is = {
      .m_buffer = (unsigned char *)sample->payload.start,
      .m_index = cdr_header_size,
      .m_size = sample->payload.len - cdr_header_size,
      .m_xcdr_version = DDSI_RTPS_CDR_ENC_VERSION_2};
  dds_stream_read(&is, (void *)&data, &dds_cdrstream_default_allocator, RoundTripModule_DataType_desc.m_ops);

  postDeserTime = dds_time();

  /* Update stats */
  difference = (postWriteTime - preWriteTime) / DDS_NSECS_IN_USEC;
  writeAccess = *exampleAddTimingToTimeStats(&writeAccess, difference);
  writeAccessOverall = *exampleAddTimingToTimeStats(&writeAccessOverall, difference);

  difference = (postDeserTime - preDeserTime) / DDS_NSECS_IN_USEC;
  readAccess = *exampleAddTimingToTimeStats(&readAccess, difference);
  readAccessOverall = *exampleAddTimingToTimeStats(&readAccessOverall, difference);

  difference = (postDeserTime - z_time_to_dds_time(sample->timestamp.time)) / DDS_NSECS_IN_USEC;
  roundTrip = *exampleAddTimingToTimeStats(&roundTrip, difference);
  roundTripOverall = *exampleAddTimingToTimeStats(&roundTripOverall, difference);

  if (!warmUp)
  {
    /* Print stats each second */
    difference = (postDeserTime - startTime) / DDS_NSECS_IN_USEC;
    if (difference > US_IN_ONE_SEC)
    {
      printf("%9" PRIi64 " %9lu %8.0f %8" PRIi64 " %8" PRIi64 " %8" PRIi64 " %10lu %8.0f %8" PRIi64 " %10lu %8.0f %8" PRIi64 "\n",
             elapsed + 1,
             roundTrip.count,
             exampleGetMedianFromTimeStats(&roundTrip) / 2,
             roundTrip.min / 2,
             exampleGet99PercentileFromTimeStats(&roundTrip) / 2,
             roundTrip.max / 2,
             writeAccess.count,
             exampleGetMedianFromTimeStats(&writeAccess),
             writeAccess.min,
             readAccess.count,
             exampleGetMedianFromTimeStats(&readAccess),
             readAccess.min);
      fflush(stdout);

      exampleResetTimeStats(&roundTrip);
      exampleResetTimeStats(&writeAccess);
      exampleResetTimeStats(&readAccess);
      startTime = dds_time();
      elapsed++;
    }
  }

  /* Send ping again */
  send_ping();
}

static void usage(void)
{
  printf("Usage (parameters must be supplied in order):\n"
         "./z_ping [payloadSize (bytes, 0 - 100M)] [numSamples (0 = infinite)] [timeOut (seconds, 0 = infinite)] [zenoh_locator]\n"
         "Defaults:\n"
         "./z_ping 0 0 0 tcp/127.0.0.1:7447\n");
  exit(EXIT_FAILURE);
}

int main(int argc, char *argv[])
{
  uint32_t payloadSize = 0;
  uint64_t numSamples = 0;
  bool invalidargs = false;
  const char *locator = "tcp/127.0.0.1:7447";
  dds_time_t timeOut = 0;

  unsigned long i;

  int argidx = 1;
  if (argc - argidx == 0)
  {
    invalidargs = true;
  }
  if (argc - argidx >= 1)
  {
    payloadSize = (uint32_t)atol(argv[argidx]);

    if (payloadSize > 100 * 1048576)
    {
      invalidargs = true;
    }
  }
  if (argc - argidx >= 2)
  {
    numSamples = (uint64_t)atol(argv[argidx + 1]);
  }
  if (argc - argidx >= 3)
  {
    timeOut = atol(argv[argidx + 2]);
  }
  if (argc - argidx >= 4)
  {
    locator = argv[argidx + 3];
  }
  if (invalidargs || (argc - argidx == 1 && (strcmp(argv[argidx], "-h") == 0 || strcmp(argv[argidx], "--help") == 0)))
    usage();

  exampleInitTimeStats(&roundTrip);
  exampleInitTimeStats(&writeAccess);
  exampleInitTimeStats(&readAccess);
  exampleInitTimeStats(&roundTripOverall);
  exampleInitTimeStats(&writeAccessOverall);
  exampleInitTimeStats(&readAccessOverall);

  memset(&pub_data, 0, sizeof(pub_data));

  z_owned_config_t config = z_config_default();
  zp_config_insert(z_config_loan(&config), Z_CONFIG_MODE_KEY, z_string_make("client"));
  zp_config_insert(z_config_loan(&config), Z_CONFIG_PEER_KEY, z_string_make(locator));

  printf("=== Z_FRAG_MAX_SIZE = %d\n", Z_FRAG_MAX_SIZE);
  printf("=== Z_BATCH_SIZE_TX = %d\n", Z_BATCH_SIZE_TX);
  printf("Opening session, connecting to %s ...\n", locator);
  session = z_open(z_config_move(&config));
  if (!z_session_check(&session))
  {
    printf("Unable to open session!\n");
    return -1;
  }

  // Start read and lease tasks for zenoh-pico
  if (zp_start_read_task(z_session_loan(&session), NULL) < 0 || zp_start_lease_task(z_session_loan(&session), NULL) < 0)
  {
    printf("Unable to start read and lease tasks");
    return -1;
  }

  // Declare publisher on "ping/RoundTrip"
  pub = z_declare_publisher(z_session_loan(&session), z_keyexpr(PING_KEY_EXPR), NULL);
  if (!z_publisher_check(&pub))
  {
    printf("Unable to declare publisher for key expression!\n");
    return -1;
  }

  // Declare subscriber on "pong/RoundTrip"
  z_owned_closure_sample_t callback = z_closure_sample(data_handler, NULL, NULL);
  sub =
      z_declare_subscriber(z_session_loan(&session), z_keyexpr(PONG_KEY_EXPR), z_closure_sample_move(&callback), NULL);
  if (!z_subscriber_check(&sub))
  {
    printf("Unable to declare subscriber.\n");
    return -1;
  }

  printf("# payloadSize: %" PRIu32 " | numSamples: %" PRIu64 " | timeOut: %" PRIi64 "\n\n", payloadSize, numSamples, timeOut);
  fflush(stdout);

  pub_data.payload._length = payloadSize;
  pub_data.payload._buffer = payloadSize ? malloc(payloadSize) : NULL;
  pub_data.payload._release = true;
  pub_data.payload._maximum = 0;
  for (i = 0; i < payloadSize; i++)
  {
    pub_data.payload._buffer[i] = 'a';
  }

  startTime = dds_time();
  /* Send 1st ping */
  send_ping();

  printf("# Waiting for startup jitter to stabilise\n");
  fflush(stdout);
  z_sleep_ms(WARMUP_TIME_MS);

  warmUp = false;
  printf("# Warm up complete.\n\n");
  printf("# Latency measurements (in us)\n");
  printf("#             Latency [us]                                   Write-access time [us]       Read-access time [us]\n");
  printf("# Seconds     Count   median      min      99%%      max      Count   median      min      Count   median      min\n");
  fflush(stdout);

  exampleResetTimeStats(&roundTrip);
  exampleResetTimeStats(&writeAccess);
  exampleResetTimeStats(&readAccess);
  for (i = 0; (!numSamples || i < numSamples) && !(timeOut && elapsed >= timeOut); i++)
  {
    z_sleep_ms(200);
  }

  printf(
      "\n%9s %9lu %8.0f %8" PRIi64 " %8" PRIi64 " %8" PRIi64 " %10lu %8.0f %8" PRIi64 " %10lu %8.0f %8" PRIi64 "\n",
      "# Overall",
      roundTripOverall.count,
      exampleGetMedianFromTimeStats(&roundTripOverall) / 2,
      roundTripOverall.min / 2,
      exampleGet99PercentileFromTimeStats(&roundTripOverall) / 2,
      roundTripOverall.max / 2,
      writeAccessOverall.count,
      exampleGetMedianFromTimeStats(&writeAccessOverall),
      writeAccessOverall.min,
      readAccessOverall.count,
      exampleGetMedianFromTimeStats(&readAccessOverall),
      readAccessOverall.min);
  fflush(stdout);

  z_undeclare_subscriber(z_subscriber_move(&sub));

  // Stop read and lease tasks for zenoh-pico
  zp_stop_read_task(z_session_loan(&session));
  zp_stop_lease_task(z_session_loan(&session));

  z_close(z_session_move(&session));

  /* Clean up */
  exampleDeleteTimeStats(&roundTrip);
  exampleDeleteTimeStats(&writeAccess);
  exampleDeleteTimeStats(&readAccess);
  exampleDeleteTimeStats(&roundTripOverall);
  exampleDeleteTimeStats(&writeAccessOverall);
  exampleDeleteTimeStats(&readAccessOverall);

  return EXIT_SUCCESS;
}
