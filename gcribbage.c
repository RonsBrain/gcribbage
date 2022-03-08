#include "application.h"

int main(int argc, char **argv) {
  return g_application_run(G_APPLICATION(gcribbage_application_new()), argc,
                           argv);
}
