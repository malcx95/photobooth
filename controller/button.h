#pragma once

#include <stdint.h>
#include <Arduino.h>

namespace button
{

enum class Button : uint8_t
{
  ACCEPT = 6,
  REJECT = 7,
  TAKE_PHOTO = 8,
};

struct ButtonState
{
  Button button;
  uint8_t led_pin;
  bool pressed;
  bool prev_pressed;

  void setup();
  bool read_state();
  void update_state();
};

}
