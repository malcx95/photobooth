#pragma once

#include <FastLED.h>
#include <stdint.h>

constexpr size_t NUM_LEDS = 160 * 5;

enum class LEDState
{
  STANDBY,
  COUNTDOWN_1,
  COUNTDOWN_2,
  COUNTDOWN_3
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
