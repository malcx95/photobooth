#include "program.h"
#include "communication.h"
#include <Arduino.h>


constexpr uint8_t RGB_PIN = 10;

void ControllerState::init()
{
  Serial.begin(9600);
  pinMode(LED_BUILTIN, OUTPUT);
  for (size_t i = 0; i < NUM_BUTTONS; ++i)
  {
    buttons[i].setup();
  }
}

void ControllerState::update()
{
  for (size_t i = 0; i < NUM_BUTTONS; ++i)
  {
    buttons[i].update_state();
  }

  comm::RXMessage msg;
  bool read = read_serial(&msg);
  if (read)
  {
    if (msg.type == comm::TURN_ON_LIGHT)
    {
      digitalWrite(13, HIGH);
      delay(1000);
    }
    else if (msg.type == comm::CONNECT)
    {
      comm::TXMessage msg{comm::CONNECT_ACK, 0};
      send_msg(&msg);
      digitalWrite(13, HIGH);
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

  digitalWrite(13, LOW);
  delay(100);
}
