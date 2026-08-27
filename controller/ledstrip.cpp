#include "ledstrip.h"
#include <cstdint>
#include <math.h>


constexpr int MAX_CURRENT_MILLIAMPS = 5000;//20000;
constexpr uint8_t RGB_PIN = 10;


void LEDStrip::init()
{
  //pinMode(LED_BUILTIN, OUTPUT);
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
    default:
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
