UI8D
====

UI8D (ui8d) is a user interface and telematics board. It is mainly targeted
towards EV conversions and for being a companion board to the IPDM56v2, but it
can be useful for other purposes also.

Hardware gotchas
================

Remote8D v2.0 (UI8D v2.0)
-------------------------

1. U305 needs to be rotated so that (when looking at it so that there's one top
   pin and two bottom pins) the top pin moves in place of the right pin, and the
   right pin moves in place of the top pin. The remaining pin that was
   originally on the left has to be connected using a bodge wire to ground,
   which can be found on the left side of RC26

2. U308 needs to be rotated so that (when looking at it so that there's one top
   pin and two bottom pins) the right pin moves in place of the top pin, the
   left pin remains in place and the top pin has to be connected to the original
   pad of the right pin by a bodge wire.

3. Remember to solder on U303 which is the 5V regulator module. Otherwise the
   board will do nothing.

MicroPython
===========

You can use micropython on UI8D.

You should compile the special board port that was made for the UI8D.

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

NOTE: Test programs are located in the ui8d repository, under test_firmware/.

print("Hello World!")

import machine
from machine import Pin

# TODO: PWM control for backlight
# TODO: Measure ambient brightness using LDR at pin PA3 and set brightness based on that
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

#import demo_color_palette
#lcd_backlight.on()
#demo_color_palette.test_palette()

np[0] = (0, 3, 1)
np.write()

while True:
    lcd_backlight.on()

