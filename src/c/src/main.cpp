#include "discovery_server.h"
#include <cstdio>
#include <cstdlib>

void participant_discovery_callback(ParticipantData *participant_data,
                                    size_t num_participants, void *data) {
  printf("participant_discovery_callback called\n");
}

int main(int argc, const char *argv[]) {
  int res = run_discovery_server_impl(1, &participant_discovery_callback);
  if (res != 0) {
    fprintf(stderr,
            "Some error happened while creating fastrtps discovery server");
    return EXIT_FAILURE;
  }
  return 0;
}