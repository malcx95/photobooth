#pragma once

#include <FastLED.h>
#include <stdint.h>

constexpr size_t NUM_LEDS = 160 * 4;

enum LEDState : uint8_t
{
  STANDBY = 1,
  COUNTDOWN_1 = 2,
  COUNTDOWN_2 = 3,
  COUNTDOWN_3 = 4,
  CAPTURING = 5,
  ERROR = 6,
  REVIEW = 7,
  LEADERBOARD = 8,
  FETCHING = 9
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
  void update_capturing();
  void update_error();
  void update_review();
  void update_leaderboard();
  void update_fetching();
private:
  CRGB leds[NUM_LEDS];
  uint64_t it;

  int16_t dir = 1;
  int16_t fetching_index = 0;
  int16_t fetching_speed_divisor = 1;
};
