#pragma once

#include <FastLED.h>
#include <stdint.h>

constexpr size_t NUM_LEDS = 160 * 5;

enum LEDState : uint8_t
{
  STANDBY = 1,
  COUNTDOWN_1 = 2,
  COUNTDOWN_2 = 3,
  COUNTDOWN_3 = 4,
  CAPTURING = 5
};

class LEDStrip
{
public:
  void init();
  void update();

  LEDState state = LEDState::STANDBY;

private:
  void update_standby();
  void update_countdown1();
  void update_countdown2();
  void update_countdown3();

private:
  CRGB leds[NUM_LEDS];
  uint64_t it;
};
