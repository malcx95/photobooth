#include "program.h"
#include "communication.h"
#include "ledstrip.h"
#include <Arduino.h>
#include <FastLED.h>


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

void ControllerState::update()
{
  for (size_t i = 0; i < NUM_BUTTONS; ++i)
  {
    buttons[i].update_state();
  }

  if (led_timer.triggered())
  {
    ledstrip.update();
  }

  if (comm_timer.triggered())
  {
    comm::RXMessage msg;
    bool read = read_serial(&msg);
    if (read)
    {
      if (msg.type == comm::TURN_ON_LIGHT)
      {
        delay(1000);
      }
      else if (msg.type == comm::CONNECT)
      {
        comm::TXMessage msg{comm::CONNECT_ACK, 0};
        send_msg(&msg);
        delay(1000);
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
