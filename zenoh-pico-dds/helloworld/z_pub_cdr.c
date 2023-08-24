
#include <ctype.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <zenoh-pico.h>
#include <HelloWorldData.h>

// CycloneDDS CDR Deserializer
#include <dds/cdr/dds_cdrstream.h>

const struct dds_cdrstream_allocator dds_cdrstream_default_allocator = {malloc, realloc, free};

// CDR Xtypes header {0x00, 0x01} indicates it's Little Endian (CDR_LE representation)
const uint8_t cdr_header[4] = {0x00, 0x01, 0x00, 0x00};

const size_t alloc_size = 4096; // Abitrary size

long int get_timestamp()
{
    struct timeval time;
    int res = gettimeofday(&time, NULL);
    if (res != 0)
    {
        printf("publisher get timestamp error!\n");
        return 0;
    }
    return (time.tv_sec * 1000000 + time.tv_usec);
}

int main(int argc, char **argv)
{
    const char *keyexpr = "HelloWorldData_Msg";
    const char *mode = "client";
    char *locator = NULL;

    int opt;
    while ((opt = getopt(argc, argv, "k:e:m:")) != -1)
    {
        switch (opt)
        {
        case 'k':
            keyexpr = optarg;
            break;
        case 'e':
            locator = optarg;
            break;
        case 'm':
            mode = optarg;
            break;
        case '?':
            if (optopt == 'k' || optopt == 'v' || optopt == 'e' || optopt == 'm')
            {
                fprintf(stderr, "Option -%c requires an argument.\n", optopt);
            }
            else
            {
                fprintf(stderr, "Unknown option `-%c'.\n", optopt);
            }
            return 1;
        default:
            return -1;
        }
    }

    // Set HelloWorld IDL message
    HelloWorldData_Msg msg;
    msg.userID = 1;
    msg.message = "Hello World from zenoh-pico!";

    z_owned_config_t config = z_config_default();
    zp_config_insert(z_config_loan(&config), Z_CONFIG_MODE_KEY, z_string_make(mode));
    if (locator != NULL)
    {
        zp_config_insert(z_config_loan(&config), Z_CONFIG_PEER_KEY, z_string_make(locator));
    }

    printf("Opening session...\n");
    z_owned_session_t s = z_open(z_config_move(&config));
    if (!z_session_check(&s))
    {
        printf("Unable to open session!\n");
        return -1;
    }

    // Start read and lease tasks for zenoh-pico
    if (zp_start_read_task(z_session_loan(&s), NULL) < 0 || zp_start_lease_task(z_session_loan(&s), NULL) < 0)
    {
        printf("Unable to start read and lease tasks");
        return -1;
    }

    printf("Declaring publisher for '%s'...\n", keyexpr);
    z_owned_publisher_t pub = z_declare_publisher(z_session_loan(&s), z_keyexpr(keyexpr), NULL);
    if (!z_publisher_check(&pub))
    {
        printf("Unable to declare publisher for key expression!\n");
        return -1;
    }

    // Setup ostream for serializer
    dds_ostream_t os;
    struct dds_cdrstream_desc desc;

    // Allocate buffer for serialized message
    uint8_t *buf = malloc(alloc_size);

    for (int idx = 0; idx < 1; ++idx)
    {
        sleep(1);
        printf("Putting Data ('%s')...\n", keyexpr);

        // Add ROS2 header
        memcpy(buf, cdr_header, sizeof(cdr_header));

        os.m_buffer = buf;
        os.m_index = sizeof(cdr_header); // Offset for CDR Xtypes header
        os.m_size = alloc_size;
        os.m_xcdr_version = DDSI_RTPS_CDR_ENC_VERSION_2;

        // Do serialization
        bool ret = dds_stream_write(&os, &dds_cdrstream_default_allocator,
                                    (void *)&msg, HelloWorldData_Msg_desc.m_ops);

        if (ret == true)
        {
            z_publisher_put_options_t options = z_publisher_put_options_default();
            options.encoding = z_encoding(Z_ENCODING_PREFIX_TEXT_PLAIN, NULL);
            z_publisher_put(z_publisher_loan(&pub), (const uint8_t *)buf, os.m_index, &options);
        }
    }

    z_undeclare_publisher(z_publisher_move(&pub));

    // Stop read and lease tasks for zenoh-pico
    zp_stop_read_task(z_session_loan(&s));
    zp_stop_lease_task(z_session_loan(&s));

    z_close(z_session_move(&s));

    return 0;
}
