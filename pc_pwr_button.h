#include <HomeSpan.h>

struct PCPwrButton : Service::Switch {
  SpanCharacteristic *on;

  int pinRelayOut;
  int pinPwrBtnIn;
  int pinDebug;

  PCPwrButton(int pinPwrBtnIn, int pinRelayOut, int pinDebug)
    : Service::Switch() {
    on = new Characteristic::On();

    this->pinPwrBtnIn = pinPwrBtnIn;
    pinMode(pinPwrBtnIn, INPUT);
    new SpanButton(pinPwrBtnIn);

    this->pinRelayOut = pinRelayOut;
    pinMode(pinRelayOut, OUTPUT);

    this->pinDebug = pinDebug;
    pinMode(pinDebug, OUTPUT);
  }

  boolean update() {
    int v = on->getNewVal();
    if (v == 1) {
      pushRelayOut();
    }
    return (true);
  }

  void button(int pin, int pressType) override {
    if (pin == pinPwrBtnIn) {
      if (pressType == SpanButton::SINGLE) {
        if (on->getVal() == 0) {
          on->setVal(1);
          pushRelayOut();
        }
      } else if (pressType == SpanButton::DOUBLE) {
        // TBU
      } else if (pressType == SpanButton::LONG) {
        // TBU
      }
    }
  }

  void pushRelayOut() {
    digitalWrite(this->pinDebug, 1);
    digitalWrite(this->pinRelayOut, 1);
    delay(500);
    on->setVal(0);
    digitalWrite(this->pinRelayOut, 0);
    digitalWrite(this->pinDebug, 0);
  }
};