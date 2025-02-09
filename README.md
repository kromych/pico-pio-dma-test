# Experiments with PIO and DMA

Logs to the UART. The cable that I have should be connected to
the UART0 pins on the Pico. The pins are GPIO0 and GPIO1, and
the wires are BLUE -> GPIO0, GREEN -> GPIO1, and BLACK -> GND.

```sh
picocom -b 115200 -f n -d 8 -s 1 /dev/tty.usbmodem84102  # macOS
```
