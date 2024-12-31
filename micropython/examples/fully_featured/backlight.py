import machine
from machine import Pin, SPI
import time
from pyb import Timer, ADC
import portable_asyncio as asyncio

# PWM control for backlight
# TODO: Measure ambient brightness using LDR at pin PA3 and set brightness based on that
lcd_backlight_pin = Pin.board.PD12
lcd_backlight_timer = Timer(4, freq=1000)
lcd_backlight_channel = lcd_backlight_timer.channel(1, Timer.PWM, pin=lcd_backlight_pin, pulse_width_percent=50)

ldr_pin = Pin.board.PA3
ldr_adc = ADC(ldr_pin)

def set_brightness_percent(percent):
    lcd_backlight_channel.pulse_width_percent(percent)

def get_ldr_percent():
    return ldr_adc.read() / 4095.0 * 100.0

def update_brightness_based_on_ldr():
    set_brightness_percent(max(get_ldr_percent(), 1))

# You can create an asyncio task out of this to continuously update backlight brightness in the background
async def update_task():
    while True:
        update_brightness_based_on_ldr()
        await asyncio.sleep_ms(2000)

