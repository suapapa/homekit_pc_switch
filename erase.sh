~/.espressif/python_env/idf5.4_py3.14_env/bin/esptool.py \
  --chip esp32s2 \
  --port /dev/cu.usbmodem01 \
  --before usb_reset \
  erase_flash
