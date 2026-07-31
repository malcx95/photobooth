#include "program.h"

ControllerState state;

void setup()
{
  state.init();
}

void loop()
{
  state.update();
}
