#include "RoundTrip.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>
#include <stdlib.h>
#include <zenoh-pico.h>

/* The Zenoh key expr that zenoh-bridge-dds will map to DDS Topics (format: <partition>/<topic>) */
#define PING_KEY_EXPR "ping/RoundTrip"
#define PONG_KEY_EXPR "pong/RoundTrip"

static z_owned_session_t session;
static z_owned_publisher_t pub;
static z_owned_subscriber_t sub;

void data_handler(const z_sample_t *sample, void *arg)
{
  (void)arg;
  printf(".");
  fflush(stdout);

  /* send payload back to ping */
  z_publisher_put_options_t options = z_publisher_put_options_default();
  z_publisher_put(z_publisher_loan(&pub), sample->payload.start, sample->payload.len, &options);
}

int main(int argc, char *argv[])
{
  const char *locator = "tcp/127.0.0.1:7447";

  int argidx = 1;
  if (argc - argidx >= 1)
  {
    locator = argv[argidx];
  }

  z_owned_config_t config = z_config_default();
  zp_config_insert(z_config_loan(&config), Z_CONFIG_MODE_KEY, z_string_make("client"));
  zp_config_insert(z_config_loan(&config), Z_CONFIG_PEER_KEY, z_string_make(locator));

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

  // Declare subscriber on "ping/RoundTrip"
  z_owned_closure_sample_t callback = z_closure_sample(data_handler, NULL, NULL);
  sub =
      z_declare_subscriber(z_session_loan(&session), z_keyexpr(PING_KEY_EXPR), z_closure_sample_move(&callback), NULL);
  if (!z_subscriber_check(&sub))
  {
    printf("Unable to declare subscriber.\n");
    return -1;
  }

  // Declare publisher on "pong/RoundTrip"
  pub = z_declare_publisher(z_session_loan(&session), z_keyexpr(PONG_KEY_EXPR), NULL);
  if (!z_publisher_check(&pub))
  {
    printf("Unable to declare publisher for key expression!\n");
    return -1;
  }

  printf("Waiting for samples from ping to send back... (Enter 'q' to quit)\n");
  fflush(stdout);

  char c = '\0';
  while (c != 'q')
  {
    fflush(stdin);
    scanf("%c", &c);
  }

  z_undeclare_subscriber(z_subscriber_move(&sub));

  // Stop read and lease tasks for zenoh-pico
  zp_stop_read_task(z_session_loan(&session));
  zp_stop_lease_task(z_session_loan(&session));

  z_close(z_session_move(&session));

  return EXIT_SUCCESS;
}
