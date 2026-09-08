#include "communication.h"
#include <Arduino.h>

bool comm::read_serial(comm::RXMessage* msg)
{
  if (Serial.available() >= 3)
  {
    bool found = false;
    while (!found)
    {
      found = Serial.read() == RX_START_WORD;
    }
    if (!found)
    {
      return false;
    }

    Serial.readBytes(reinterpret_cast<char*>(msg), sizeof(comm::RXMessage));
    clear_serial();
    return true;
  }
  else
  {
    return false;
  }
}

void comm::send_msg(comm::TXMessage* msg)
{
  Serial.write(comm::TX_START_WORD);
  Serial.flush();
  Serial.write(reinterpret_cast<char*>(msg), sizeof(comm::TXMessage));
  Serial.flush();
  comm::clear_serial();
}

void comm::clear_serial()
{
  while (Serial.available() > 0)
  {
    Serial.read();
  }
}
