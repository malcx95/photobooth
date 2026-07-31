#include "button.h"

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
  digitalWrite(led_pin, pressed ? HIGH : LOW);
}
