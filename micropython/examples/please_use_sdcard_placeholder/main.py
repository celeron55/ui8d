print("UI8D v2.0")

import machine
from machine import Pin, SPI
import time
from pyb import Timer

import asyncio
import backlight

import neopixel
statusled = neopixel.NeoPixel(Pin.board.PA6, 1)
statusled[0] = (4, 0, 0)
statusled.write()

from pyb import CAN
can = CAN(1, CAN.LOOPBACK)
can.setfilter(0, CAN.LIST16, 0, (123, 124, 125, 126))  
can.send('message!', 123)
can.recv(0)

statusled[0] = (0, 1, 3)
statusled.write()

from ili9341 import Display, color565
# Yes it says 1 GHz. It's just picking the highest possible rate.
spi = SPI(3, baudrate=1000000000)
display = Display(spi, dc=Pin.board.PD14, cs=Pin.board.PD11, rst=Pin.board.PD13)

display.clear(color565(0, 0, 0))

from xglcd_font import XglcdFont
print('Loading fonts')
font_unispace = XglcdFont('fonts/Unispace12x24.c', 12, 24)

def draw_ui_text(self, x, y, text, color, align='L'):
    text_w = 0
    if align == 'L':
        text_w = font_unispace.measure_text(text)
    self.draw_text(240 - 24 - y, x + text_w, text, font_unispace, color, landscape=True, rotate_180=True)
setattr(Display, "draw_ui_text", draw_ui_text)

#display.draw_text8x8(240-20, 240-35, 'Remote8D v2.0', color565(255, 0, 255), rotate=90)
display.draw_ui_text(319, 0, 'Remote8D v2.0', color565(255, 0, 255), align='R')

statusled[0] = (0, 3, 1)
statusled.write()

#display.draw_ui_text(0, 50, "Battery: 75%  25 C  3.95 V", color565(255, 255, 255))
#display.draw_ui_text(0, 75, "Heater: 53 C", color565(255, 255, 255))
#display.draw_ui_text(0, 100, "System: 14.0 V", color565(255, 255, 255))

display.draw_ui_text(0, 100, "Please use SD card", color565(255, 200, 50))

async def display_task():
    i = 0
    while True:
        display.draw_ui_text(0, 0, str(i), color565(255, 0, 255))
        i += 1
        await asyncio.sleep_ms(1000)

asyncio.run(asyncio.gather(
    asyncio.create_task(backlight.update_task()),
    asyncio.create_task(display_task())
))


