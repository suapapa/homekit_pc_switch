#include <HomeSpan.h>
#include "pc_pwr_button.h"

#define PIN_STATUS_LED 15
#define PIN_PWRBTN_IN 18
#define PIN_RELAY_OUT 12
#define PIN_DEBUG 16

void setup() {
  Serial.begin(115200);

  homeSpan.setStatusPin(PIN_STATUS_LED);
  homeSpan.setPairingCode("11122334");
  homeSpan.setQRID("111-22-334");

  homeSpan.begin(Category::Switches, "PC Power Button");

  new SpanAccessory();
    new Service::AccessoryInformation();
      new Characteristic::Identify();
      new Characteristic::Name("PC Power Button");
    new PCPwrButton(PIN_PWRBTN_IN, PIN_RELAY_OUT, PIN_DEBUG);
}

void loop() {
  homeSpan.poll();
}