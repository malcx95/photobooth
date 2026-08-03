#pragma once

#include <FastLED.h>
#include "button.h"
#include "ledstrip.h"
#include "timer.h"

constexpr size_t NUM_BUTTONS = 3;

class ControllerState
{
public:
  void init();
  void update();

private:

  button::ButtonState buttons[NUM_BUTTONS]
  {
    {button::Button::ACCEPT, 5, false, false},
    {button::Button::REJECT, 4, false, false},
    {button::Button::TAKE_PHOTO, 3, false, false},
  };

  LEDStrip ledstrip;
  Timer comm_timer{100};
  Timer led_timer{10};

};
