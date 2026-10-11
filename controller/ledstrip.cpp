#include "ledstrip.h"
#include <cstdint>
#include <math.h>


constexpr int MAX_CURRENT_MILLIAMPS = 5000;//20000;
constexpr uint8_t RGB_PIN = 10;


void LEDStrip::init()
{
  FastLED.addLeds<WS2812B, RGB_PIN, GRB>(leds, NUM_LEDS);
  FastLED.setBrightness(255);
  FastLED.setMaxPowerInVoltsAndMilliamps(5, MAX_CURRENT_MILLIAMPS);
}

void LEDStrip::update()
{
  switch (state) {
    case LEDState::STANDBY:
      update_standby();
      break;
    case LEDState::COUNTDOWN_1:
      update_countdown1();
      break;
    case LEDState::COUNTDOWN_2:
      update_countdown2();
      break;
    case LEDState::COUNTDOWN_3:
      update_countdown3();
      break;
    case LEDState::CAPTURING:
      update_capturing();
      break;
    case LEDState::ERROR:
      update_error();
      break;
    case LEDState::REVIEW:
      update_review();
      break;
    case LEDState::LEADERBOARD:
      update_leaderboard();
      break;
    case LEDState::FETCHING:
      update_fetching();
      break;
    default:
      update_standby();
      break;
  }

  it++;
  FastLED.show();
}


void LEDStrip::update_standby()
{
  float speed = 0.1f;
  float sparseness = 20;
  float width = 6;
  float color_speed = 0.01f;
  float color_sparseness = 0.01f;

  float it_f = it * speed;
  for (size_t i = 0; i < NUM_LEDS; ++i)
  {
    auto half_width = width / 2.f;
    auto a = fmod(((float)i + it_f), sparseness);
    auto b = fabs(a - half_width);
    float brightness = b > half_width ? 0.f : 1.f - (b / half_width);
    auto brightness_int = (uint8_t)(brightness * 255.f);

    uint8_t hue = (uint8_t)(((float)i * color_sparseness + it_f * color_speed) * 255.f);
    leds[i].setHSV(hue, 255, brightness_int);
  }
}

void LEDStrip::update_countdown1()
{
  fill_solid(leds, NUM_LEDS, CRGB::Red);
}

void LEDStrip::update_countdown2()
{
  fill_solid(leds, NUM_LEDS, CRGB::Green);
}

void LEDStrip::update_countdown3()
{
  fill_solid(leds, NUM_LEDS, CRGB::Blue);
}

void LEDStrip::update_capturing()
{
  fill_solid(leds, NUM_LEDS, CRGB::White);
}

void LEDStrip::update_error()
{
  fill_solid(leds, NUM_LEDS, CRGB::Red);
}

void LEDStrip::update_review()
{
  float speed = 0.1f;
  float sparseness = 20;
  float width = 6;
  float color_speed = 0.01f;
  float color_sparseness = 0.01f;

  float it_f = it * speed;
  for (size_t i = 0; i < NUM_LEDS; ++i)
  {
    auto half_width = width / 2.f;
    auto a = fmod(((float)i + it_f), sparseness);
    auto b = fabs(a - half_width);
    float brightness = b > half_width ? 0.f : 1.f - (b / half_width);
    auto brightness_int = (uint8_t)(brightness * 255.f);

    uint8_t hue = 45;
    leds[i].setHSV(hue, 255, brightness_int);
  }
}

void LEDStrip::update_leaderboard()
{
  fill_solid(leds, NUM_LEDS, CRGB::Black);
}

void LEDStrip::update_fetching()
{
  if (fetching_index == 0)
  {
    dir = 1;
  }
  else if (fetching_index == NUM_LEDS - 1)
  {
    dir = -1;
  }
  fetching_index += static_cast<int16_t>(it % fetching_speed_divisor == 0) * dir;
  fill_solid(leds, NUM_LEDS, CRGB::Black);
  leds[fetching_index] = CRGB::White;
}
