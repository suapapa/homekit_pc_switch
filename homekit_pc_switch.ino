#include <HomeSpan.h>
#include "pc_pwr_button.h"

#define PIN_PWRBTN_IN 18
#define PIN_RELAY_OUT 16

void setup() {
  Serial.begin(115200);

  homeSpan.setStatusPin(15);
  homeSpan.setPairingCode("11122334");
  homeSpan.setQRID("111-22-334");

  homeSpan.begin(Category::Switches, "PC Power Button");

  new SpanAccessory();
    new Service::AccessoryInformation();
      new Characteristic::Identify();
      new Characteristic::Name("PC Power Button");
    new PCPwrButton(PINT_PWR_BTN_IN, PIN_RELAY_OUT);
}

void loop() {
  homeSpan.poll();
}