#pragma once

#include <stdint.h>
#include <Arduino.h>

namespace button
{

// BIG_WHITE  = 6
// RED = 7
// BLUE = 8
// WHITE = 9
// YELLOW = 20
// GREEN = 21

enum class Button : uint8_t
{
  BIG_WHITE  = 6,
  RED = 7,
  BLUE = 8,
  WHITE = 9,
  YELLOW = 20,
  GREEN = 21,
};

struct ButtonState
{
  Button button;
  uint8_t led_pin;
  bool pressed;
  bool prev_pressed;
  bool enabled;

  void setup();
  bool read_state();
  void update_state();
  void update_led();
  void set_brightness(float brightness);
};

}
