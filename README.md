UI8D
====

UI8D (ui8d) is a user interface and telematics board. It is mainly targeted
towards EV conversions and for being a companion board to the IPDM56v2, but it
can be useful for other purposes also.

The software runs on an STM32F4 and it can host different modules:
- ILI9341 (LCD)
- SIM7600 (LTE)
- RFM95 (LoRa)
- W5500 (Ethernet)

It can be programmed in multiple programming languages, including:
- Rust
- MicroPython
- C++ (*)
- C (*)

(*) No example software provided

Hardware gotchas
================

UI8D v2.0 (also known as Remote8D v2.0)
---------------------------------------

1. U305 needs to be rotated so that (when looking at it so that there's one top
   pin and two bottom pins) the top pin moves in place of the right pin, and the
   right pin moves in place of the top pin. The remaining pin that was
   originally on the left has to be connected using a bodge wire to ground,
   which can be the original pad of the left pin, or the bottom end of R303.

2. U308 needs to be rotated so that (when looking at it so that there's one top
   pin and two bottom pins) the right pin moves in place of the top pin, the
   left pin remains in place and the top pin has to be connected to the original
   pad of the right pin by a bodge wire.

3. Remember to solder on U303 which is the 5V regulator module. Otherwise the
   board will do nothing.

4. If you have a board where U302 was replaced with SP3223E (it should be an
   SP3222E): In order to use the RS232 connection towards the main board, you
   need to connect U302 pin 14 (ONLINE#) to C310 positive side.
	* In order to check whether the U302 charge pump is working, measure from
	  GND to C308 bottom side (5.7V) and C309 top side (-5.7V), and the TX pin
	  towards the main board should idle at -5.7V.

5. MicroPython on STM32F4 has very little program space in the flash. You may
   want to format a Micro SD card with the fat32 filesystem and insert it into
   the slot. This way you will have practically infinite program space and will
   run out of RAM first. The SD card storage is visible via USB.
	* E.g. `sudo mkfs.fat -F 32 /dev/mmcblk0p1`

6. When pressing BUTTON1 on a bare board, beware of the reset pin on the pin
   header at the corner. You are very likely to accidentally reset the board
   when touching the corner.

7. The PWMOUT1 and PWMOUT2 outputs are driven by EG3001 MOSFET gate drivers that
   will fail if the output is loaded with anything more than a capactive load of
   1nF or so, i.e. they are signal outputs only. If you want to use them _at
   all_, you should replace the 200mA PPTCs with 1k 1206 resistors. This is
   _untested_ and may or may not provide enough protection for the drivers.

Rust example
============

Rust is the recommended programming language, as it allows you to cram way more,
way faster code onto the STM32.

See https://github.com/celeron55/ui8d/tree/master/rust/ui8drust

MicroPython port
================

The MicroPython port for this board is available at
https://github.com/celeron55/ui8d_micropython.git

You can use micropython on UI8D.

You should compile the special board port that was made for the UI8D.

Some modules have been frozen into the firmware, to enable having space left for
the actual program. Here is the current list:
- ili9341.py

Micropython's stm32f407 discovery firmware kind of works, but it has unnecessary
and annoying restrictions like SPI3 and PB3 not being available (needed for the
LCD). 

Compiling:
$ git clone https://github.com/celeron55/ui8d_micropython.git
$ cd ui8d_micropython/ports/stm32
$ vim boards/STM32F407_UI8DV20/mpconfigboard.h
$ make -j6 BOARD=STM32F407_UI8DV20
$ dfu-util -D build-STM32F407_UI8DV20/firmware.dfu

Plug in an SD card into the slot on the Remote8D board. The board will run code
from the SD card, and when plugged into USB, the board will display the SD card
contents instead of interna flash. This allows fitting much more code and
resources for your program and you can update the program outside of the car, by
taking the SD card with you. It will also work as an anti-theft device...

NOTE: Use mpremote to manage the board, OR picocom --baud 115200 -l -e x /dev/ttyACM0

NOTE: Install packages by copying them from https://github.com/micropython/micropython-lib/tree/master/micropython

NOTE: Test programs are located in https://github.com/celeron55/ui8d/tree/master/micropython/examples

```
print("Hello World!")

import machine
from machine import Pin

lcd_backlight = Pin.board.PD12

import neopixel
np = neopixel.NeoPixel(Pin.board.PA6, 1)
np[0] = (4, 0, 0)
np.write()

from pyb import CAN
can = CAN(1, CAN.LOOPBACK)
can.setfilter(0, CAN.LIST16, 0, (123, 124, 125, 126))  
can.send('message!', 123)
can.recv(0)

np[0] = (0, 1, 3)
np.write()

np[0] = (0, 3, 1)
np.write()

while True:
    lcd_backlight.on()
```

License
=======

See LICENSE.txt

