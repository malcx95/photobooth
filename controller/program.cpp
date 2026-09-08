#include "program.h"
#include "communication.h"
#include "ledstrip.h"
#include <Arduino.h>
#include <FastLED.h>
#include <cmath>
#include <cstdint>


void ControllerState::init()
{
  Serial.begin(9600);
  for (size_t i = 0; i < NUM_BUTTONS; ++i)
  {
    buttons[i].setup();
  }
  ledstrip.init();
  ledstrip.state = LEDState::STANDBY;
}

void ControllerState::set_enabled_buttons(uint8_t enabled_mask)
{
  for (size_t i = 0; i < NUM_BUTTONS; ++i)
  {
    bool should_be_enabled = ((1 << i) & enabled_mask) != 0;
    buttons[i].enabled = should_be_enabled;
  }
}

void ControllerState::update()
{
  if (led_timer.triggered())
  {
    ledstrip.update();
  }

  if (button_timer.triggered())
  {
    for (size_t i = 0; i < NUM_BUTTONS; ++i)
    {
      buttons[i].update_led();
    }
  }

  if (comm_timer.triggered())
  {
    for (size_t i = 0; i < NUM_BUTTONS; ++i)
    {
      buttons[i].update_state();
    }

    comm::RXMessage msg;
    bool read = read_serial(&msg);
    if (read)
    {
      if (msg.type == comm::SET_LIGHT_STATE)
      {
        ledstrip.state = (LEDState)msg.payload;
      }
      else if (msg.type == comm::CONNECT)
      {
        comm::TXMessage msg{comm::CONNECT_ACK, 0};
        send_msg(&msg);
        delay(1000);
      }
      else if (msg.type == comm::SET_ENABLED_BUTTONS)
      {
        set_enabled_buttons(msg.payload);
      }
      else
      {
        Serial.flush();
      }
    }

    for (size_t i = 0; i < NUM_BUTTONS; ++i)
    {
      button::ButtonState state = buttons[i];
      if (state.read_state())
      {
        comm::TXMessage msg{comm::BUTTON_PRESS, (uint8_t)state.button};
        send_msg(&msg);
      }
    }
  }

}
