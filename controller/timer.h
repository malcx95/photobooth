#pragma once
#include <Arduino.h>
#include <cstdint>


class Timer
{
public:
  Timer(int64_t interval) : interval{interval}
  {
    reset();
  }

  bool hasElapsed()
  {
    return (static_cast<int64_t>(millis()) - start) >= interval;
  }

  void reset()
  {
    start = static_cast<int64_t>(millis());
  }

  bool triggered()
  {
    if (hasElapsed())
    {
      reset();
      return true;
    }
    return false;
  }

private:
  int64_t interval;
  int64_t start;
};

class Blinker
{
public:
  Blinker(int64_t onTime, int64_t offTime) : onTimer{onTime}, offTimer{offTime}
  {
    onTimer.reset();
    offTimer.reset();
  }

  void reset()
  {
    state = true;
    onTimer.reset();
  }

  bool getState()
  {
    if (state)
    {
      if (onTimer.hasElapsed())
      {
        state = false;
        offTimer.reset();
      }
    }
    else
    {
      if (offTimer.hasElapsed())
      {
        state = true;
        onTimer.reset();
      }
    }
    return state;
  }

private:
  Timer onTimer;
  Timer offTimer;
  bool state;
};
