#pragma once

#include <stdint.h>

namespace comm
{

constexpr uint8_t RX_START_WORD = 0x41;
constexpr uint8_t TX_START_WORD = 0xEE;

enum RXMessageType : uint8_t
{
  CONNECT = 0xDE,
  TURN_ON_LIGHT = 0x42,
};

struct RXMessage
{
  RXMessageType type;
  uint8_t payload;
};

enum TXMessageType : uint8_t
{
  CONNECT_ACK = 0xAE,
  BUTTON_PRESS = 0xBB,
};

struct TXMessage
{
  TXMessageType type;
  uint8_t payload;
};

bool read_serial(RXMessage* msg);
void send_msg(TXMessage* msg);
void clear_serial();

}
