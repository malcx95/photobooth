#include "button.h"
#include "esp32-hal.h"
#include "math.h"

using namespace button;

void ButtonState::setup()
{
  pinMode((uint8_t)button, INPUT_PULLUP);
  pinMode(led_pin, OUTPUT);
}

bool ButtonState::read_state()
{
  return pressed && !prev_pressed;
}

void ButtonState::update_state()
{
  prev_pressed = pressed;
  pressed = digitalRead((uint8_t)button) == LOW;
  // digitalWrite(led_pin, pressed ? HIGH : LOW);
}

void ButtonState::set_brightness(float brightness)
{
  uint8_t brightness_int = (uint8_t)(fmin(fmax(brightness, 0.0), 1.0) * 255.f);
  analogWrite(led_pin, brightness_int);
}

