#include <stdint.h>

constexpr const uint8_t RX_START_WORD = 0x41;
constexpr const uint8_t TX_START_WORD = 0xEE;

enum RXMessageType : uint8_t
{
  TURN_ON_LIGHT = 0x42,
};

struct RXMessage
{
  RXMessageType type;
  uint8_t payload;
};

enum TXMessageType : uint8_t
{
  BUTTON_PRESS = 0xBB,
};

struct TXMessage
{
  TXMessageType type;
  uint8_t payload;
};

static bool read_serial(RXMessage* msg);
static void clear_serial();

void setup()
{
  Serial.begin(9600);
  // put your setup code here, to run once:
  pinMode(13, OUTPUT);
}

void loop()
{
  // put your main code here, to run repeatedly:
  RXMessage msg;
  bool read = read_serial(&msg);
  if (read)
  {
    digitalWrite(13, HIGH);
    delay(100);
    digitalWrite(13, LOW);
    delay(100);
    digitalWrite(13, HIGH);
    delay(100);
    digitalWrite(13, LOW);

    if (msg.type == TURN_ON_LIGHT)
    {
      digitalWrite(13, HIGH);
      delay(1000);
    }
  }
  digitalWrite(13, LOW);
  delay(100);
}

static bool read_serial(RXMessage* msg)
{
  if (Serial.available() >= 3)
  {
    bool found = false;
    while (!found && Serial.available() > 3)
    {
      found = Serial.read() == RX_START_WORD;
    }
    if (!found)
    {
      return false;
    }

    Serial.readBytes(reinterpret_cast<char*>(msg), sizeof(msg));
    clear_serial();
    return true;
  }
  else
  {
    return false;
  }
}

static void clear_serial()
{
  while (Serial.available() > 0)
  {
    Serial.read();
  }
}
