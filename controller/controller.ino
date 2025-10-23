#include <stdint.h>

constexpr const uint8_t RX_START_WORD = 0x41;
constexpr const uint8_t TX_START_WORD = 0xEE;

enum class Button : uint8_t
{
  LEFT = 6,
  TAKE_PHOTO = 5,
  RIGHT = 4,
};

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

struct ButtonState
{
  Button button;
  bool pressed;
  bool prev_pressed;

  void setup()
  {
    pinMode((uint8_t)button, INPUT_PULLUP);
  }

  bool read_state()
  {
    return pressed && !prev_pressed;
  }

  void update_state()
  {
    prev_pressed = pressed;
    pressed = digitalRead((uint8_t)button) == LOW;
  }
};

ButtonState buttons[]
{
  {Button::LEFT, false, false},
  {Button::TAKE_PHOTO, false, false},
  {Button::RIGHT, false, false},
};

const size_t NUM_BUTTONS = 3;

static bool read_serial(RXMessage* msg);
static void send_msg(TXMessage* msg);
static void clear_serial();
static void setup_buttons();
static void update_buttons();

void setup()
{
  Serial.begin(9600);
  pinMode(13, OUTPUT);
  setup_buttons();
}

void loop()
{
  update_buttons();

  // if (Serial.available() > 0)
  // {
  //   digitalWrite(13, HIGH);
  //   delay(100);
  //   digitalWrite(13, LOW);
  //   delay(100);
  //   digitalWrite(13, HIGH);
  // }
  RXMessage msg;
  bool read = read_serial(&msg);
  if (read)
  {

    // digitalWrite(13, HIGH);
    // delay(100);
    // digitalWrite(13, LOW);
    // delay(10);
    // digitalWrite(13, HIGH);
    // delay(100);
    // digitalWrite(13, LOW);

    if (msg.type == TURN_ON_LIGHT)
    {
      digitalWrite(13, HIGH);
      delay(1000);
    }
    else if (msg.type == CONNECT)
    {
      TXMessage msg{CONNECT_ACK, 0};
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
    ButtonState state = buttons[i];
    if (state.read_state())
    {
      TXMessage msg{BUTTON_PRESS, (uint8_t)state.button};
      send_msg(&msg);
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
    while (!found/* && Serial.available() > 3*/)
    {
      found = Serial.read() == RX_START_WORD;
    }
    if (!found)
    {
      return false;
    }

    Serial.readBytes(reinterpret_cast<char*>(msg), sizeof(RXMessage));
    clear_serial();
    return true;
  }
  else
  {
    return false;
  }
}

static void send_msg(TXMessage* msg)
{
  Serial.write(TX_START_WORD);
  Serial.flush();
  Serial.write(reinterpret_cast<char*>(msg), sizeof(TXMessage));
  Serial.flush();
  clear_serial();
}

static void clear_serial()
{
  while (Serial.available() > 0)
  {
    Serial.read();
  }
}

static void setup_buttons()
{
  for (size_t i = 0; i < NUM_BUTTONS; ++i)
  {
    buttons[i].setup();
  }
}

static void update_buttons()
{
  for (size_t i = 0; i < NUM_BUTTONS; ++i)
  {
    buttons[i].update_state();
  }
}
