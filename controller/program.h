#pragma once

#include <FastLED.h>
#include "button.h"
#include "ledstrip.h"
#include "timer.h"

constexpr size_t NUM_BUTTONS = 6;

class ControllerState
{
public:
  void init();
  void update();

private:

  // BIG_WHITE  = 5
  // RED =        4
  // BLUE =       3
  // WHITE =      2
  // YELLOW =     1
  // GREEN =      0

  button::ButtonState buttons[NUM_BUTTONS]
  {
    {button::Button::BIG_WHITE, 5, false, false, false},
    {button::Button::RED,       4, false, false, false},
    {button::Button::BLUE,      3, false, false, false},
    {button::Button::WHITE,     2, false, false, false},
    {button::Button::YELLOW,    1, false, false, false},
    {button::Button::GREEN,     0, false, false, false},
  };

  LEDStrip ledstrip;
  Timer comm_timer{10};
  Timer led_timer{50};
  Timer button_timer{50};

private:
  void set_enabled_buttons(uint8_t enabled_mask);
};

