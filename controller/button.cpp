#include "button.h"
#include "esp32-hal.h"
#include "math.h"

using namespace button;

void ButtonState::setup()
{
  pinMode((uint8_t)button, INPUT_PULLUP);
  pinMode(led_pin, OUTPUT);
  pressed = false;
  prev_pressed = false;
}

bool ButtonState::read_state()
{
  return pressed && !prev_pressed;
}

void ButtonState::update_led()
{
  if (!enabled)
  {
    set_brightness(0.f);
  }
  else if (pressed)
  {
    set_brightness(1.f);
  }
  else
  {
    const auto it = millis();
    const float brightness = 0.1 + 0.6 * (sin((double)it / 500.0) + 1.0) / 2.0;
    set_brightness(brightness);
  }
}

void ButtonState::update_state()
{
  prev_pressed = pressed;
  pressed = digitalRead((uint8_t)button) == LOW;
}

void ButtonState::set_brightness(float brightness)
{
  uint8_t brightness_int = (uint8_t)(fmin(fmax(brightness, 0.0), 1.0) * 255.f);
  analogWrite(led_pin, brightness_int);
}

