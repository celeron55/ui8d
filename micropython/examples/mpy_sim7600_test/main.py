print("UI8D v2.0")

import machine
from machine import Pin, SPI
import time
import utime
from pyb import Timer, ADC, UART
import sys

import asyncio
import backlight
import local_config

button_pins = [
    machine.Pin("BUTTON1", machine.Pin.IN, machine.Pin.PULL_UP),
    machine.Pin("BUTTON2", machine.Pin.IN, machine.Pin.PULL_UP),
    machine.Pin("BUTTON3", machine.Pin.IN, machine.Pin.PULL_UP),
    machine.Pin("BUTTON4", machine.Pin.IN, machine.Pin.PULL_UP),
    machine.Pin("BUTTON5", machine.Pin.IN, machine.Pin.PULL_UP),
]

sim7600_power_inhibit_pin = machine.Pin("PB9", machine.Pin.OUT)
def sim7600_power_on(on):
    if on:
        sim7600_power_inhibit_pin.low() # Active low
    else:
        sim7600_power_inhibit_pin.high()
sim7600_power_on(True)

# TODO
#button_debounce = [
#    0,
#    0,
#    0,
#    0,
#    0,
#]

event_queue = []

class ButtonPressEvent:
    def __init__(self, button_id):
        self.button_id = button_id

def button_0_pressed(pin):
    print("button 0 pressed")
    event_queue.append(ButtonPressEvent(0))
button_pins[0].irq(button_0_pressed, trigger=machine.Pin.IRQ_FALLING)

def button_1_pressed(pin):
    print("button 1 pressed")
    event_queue.append(ButtonPressEvent(1))
button_pins[1].irq(button_1_pressed, trigger=machine.Pin.IRQ_FALLING)

def button_2_pressed(pin):
    print("button 2 pressed")
    event_queue.append(ButtonPressEvent(2))
button_pins[2].irq(button_2_pressed, trigger=machine.Pin.IRQ_FALLING)

def button_3_pressed(pin):
    print("button 3 pressed")
    event_queue.append(ButtonPressEvent(3))
button_pins[3].irq(button_3_pressed, trigger=machine.Pin.IRQ_FALLING)

# OSError: IRQ resource already taken by Pin('A4')
#def button_4_pressed(pin):
#    print("button 4 pressed")
#    event_queue.append(ButtonPressEvent(4))
#button_pins[4].irq(button_4_pressed, trigger=machine.Pin.IRQ_FALLING)

vbat_measure_pin = Pin.board.PA2
vbat_measure_adc = ADC(vbat_measure_pin)

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
print('Loading fonts...')
font_unispace = XglcdFont('fonts/Unispace12x24.c', 12, 24)
print('...done.')

def draw_ui_text(self, x, y, text, color, align='L', force_pixel_length=None):
    text_w = 0
    if align == 'L':
        if force_pixel_length is not None:
            text_w = force_pixel_length
        else:
            text_w = font_unispace.measure_text(text)
    self.draw_text(240 - 24 - y, x + text_w, text, font_unispace, color,
            landscape=True, rotate_180=True, force_pixel_length=force_pixel_length)
setattr(Display, "draw_ui_text", draw_ui_text)

def draw_ui_rectangle(self, x, y, w, h, color):
    self.draw_rectangle(240 - 24 - y, x, h, w, color)
setattr(Display, "draw_ui_rectangle", draw_ui_rectangle)

def fill_ui_rectangle(self, x, y, w, h, color):
    self.fill_rectangle(240 - 24 - y, x, h, w, color)
setattr(Display, "fill_ui_rectangle", fill_ui_rectangle)

statusled[0] = (0, 3, 1)
statusled.write()


# Parameters

class Param:
    def __init__(self, name, longname, unit, expect_min, expect_max, fmt):
        self.name = name
        self.longname = longname
        self.unit = unit
        self.expect_min = expect_min
        self.expect_max = expect_max
        self.fmt = fmt
        self.value = None
        self.timeout = None

class Params:
    def __init__(self):
        self.params = {}

    def add(self, name, longname, unit='', expect_min=None, expect_max=None, fmt="{:.2f}"):
        self.params[name] = Param(name, longname, unit, expect_min, expect_max, fmt)

    def get(self, name):
        return self.params[name]

    def set_value(self, name, value, timeout=None):
        self.params[name].value = value
        self.params[name].timeout = timeout

    # Call once per second
    def update_timeouts(self):
        for param in self.params.values():
            if param.timeout is not None:
                param.timeout -= 1.0
                if param.timeout <= 0.0:
                    param.timeout = None
                    param.value = None

params = Params()
params.add("sw_version", "SW version", "", 0, 28)
params.add("aux_voltage", "Aux voltage", "V", 0, 28)
params.add("ticks_ms", "ticks_ms", "", 1, 1000000, fmt="{:.0f}")
params.add("sim_status", "SIM status")
params.add("sim_successes", "SIM successes", "", 1, 1000000, fmt="{:.0f}")
params.add("sim_failures", "SIM failures", "", 0, 100, fmt="{:.0f}")
params.add("http_checkpoint", "HTTP Checkp.", "", 10, 1000, fmt="{:.0f}")
params.add("heater_t", "Heater T", "°C", -30, 100, fmt="{:.0f}")
params.add("soc", "SoC", "%", 0, 100, fmt="{:.1f}")
params.add("cell_t_min", "Cell T min", "°C", -25, 65, fmt="{:.1f}")
params.add("cell_t_max", "Cell T max", "°C", -25, 65, fmt="{:.1f}")
params.add("cell_v_min", "Cell V min", "V", 2.95, 4.25)
params.add("cell_v_max", "Cell V max", "V", 2.95, 4.25)

# Add a large number of parameters for memory testing
#for i in range(0,200):
#    params.add("dummy_"+str(i), "Dummy "+str(i), "?", 42, 1337)

params.set_value("sw_version", "0.1")
params.set_value("ticks_ms", 0)
params.set_value("sim_successes", 0)
params.set_value("sim_failures", 0)
params.set_value("http_checkpoint", 0)

# TODO: Remove these set_values once the real sources are implemented
params.set_value("heater_t", 55, 10)
params.set_value("soc", 75)
params.set_value("cell_t_min", 28)
params.set_value("cell_t_max", 30)
params.set_value("cell_v_min", 3.95)
params.set_value("cell_v_max", 3.98)

views = {
    "main": {
        "params": [
            "sw_version",
            "ticks_ms",
            "aux_voltage",
            "sim_status",
            "sim_successes",
            "sim_failures",
            "http_checkpoint",
            #"heater_t",
            #"soc",
            #"cell_t_min",
            #"cell_t_max",
            #"cell_v_min",
            #"cell_v_max",
        ],
    },
}

view_list = ["main", "other"]
current_view_index = 0

def draw_view_labels(display, view):
    i = 0
    for param_name in view["params"]:
        param = params.get(param_name)
        display.draw_ui_text(190, 25 + i * 25, param.longname+": ", color565(150, 255, 255), align="R")
        display.draw_ui_text(260, 25 + i * 25, param.unit, color565(150, 255, 255))
        i += 1

def draw_view_values(display, view, drawn_values):
    i = 0
    for param_name in view["params"]:
        param = params.get(param_name)

        if param_name not in drawn_values or drawn_values[param_name] != param.value:
            drawn_values[param_name] = param.value
            if param.value is None:
                display.draw_ui_text(250, 25 + i * 25, "-", color565(255, 50, 50),
                        align="R", force_pixel_length=60)
            elif type(param.value) in [int, float]:
                if (param.value < param.expect_min or param.value > param.expect_max):
                    color = color565(255, 50, 50)
                else:
                    color = color565(255, 255, 150)
                display.draw_ui_text(250, 25 + i * 25, param.fmt.format(param.value), color, align="R")
            else:
                display.draw_ui_text(250, 25 + i * 25, str(param.value),
                        color565(255, 255, 150), align="R", force_pixel_length=60)
                
        i += 1

async def display_task():
    global current_view_index, view_list

    drawn_view_name = None
    view = None
    drawn_values = {}

    i = 0
    while True:
        view_name = view_list[current_view_index]
        if drawn_view_name != view_name:
            drawn_view_name = view_name
            drawn_values = {}
            view = views[view_name]
            display.clear(color565(0, 0, 0))
            display.draw_ui_text(319, 0, 'UI8D v2.0', color565(255, 0, 255), align='R')
            display.draw_ui_text(100, 0, "View "+str(current_view_index+1)+"/"+str(len(view_list)), color565(255, 0, 255))
            draw_view_labels(display, view)
        display.draw_ui_text(0, 0, str(i), color565(255, 0, 255))
        draw_view_values(display, view, drawn_values)
        i += 1
        await asyncio.sleep_ms(200)

async def event_task():
    global current_view_index, view_list

    while True:
        if len(event_queue) == 0:
            await asyncio.sleep_ms(20)
            continue
            
        event = event_queue.pop(0)

        if type(event) == ButtonPressEvent:
            if event.button_id == 0:
                current_view_index -= 1
                if current_view_index < 0:
                    current_view_index = len(view_list) - 1
            elif event.button_id == 1:
                current_view_index += 1
                if current_view_index >= len(view_list):
                    current_view_index = 0
            elif event.button_id == 2:
                sys.exit() # Soft reset

async def adc_task():
    while True:
        params.update_timeouts()
        
        vbat_measure_avg = 0
        
        for i in range(0,10):
            vbat_measure_avg += vbat_measure_adc.read()

            await asyncio.sleep_ms(100)
        
        vbat_measure_avg /= 10
        
        params.set_value("aux_voltage", vbat_measure_avg / 4095.0 * 3.3 * 110.0 / 10.0, 5)

async def params_task():
    while True:
        params.update_timeouts()
        
        params.set_value("ticks_ms", utime.ticks_ms())
        
        # TODO: For testing. Remove when testing isn't needed
        params.set_value("heater_t", params.get("heater_t").value + 1.0, 10)
        
        await asyncio.sleep_ms(1000)


# SIM7600

from sim7600 import Sim

params.set_value("sim_status", "init")

print("SIM7600: UART")
sim_uart = UART(2, 115200, timeout=1000, read_buf_len=1000)
print("SIM7600: Sim")
sim = Sim(sim_uart)

async def sim_task():
    while True:
        try:
            print("sim_task: HTTP request")
            params.set_value("sim_status", "req")
            response = await sim.http_get("{:s}?ticks_ms={:d}&successes={:d}&failures={:d}&last_http_checkpoint={:d}".format(local_config.GET_URL_PREFIX, utime.ticks_ms(), params.get("sim_successes").value, params.get("sim_failures").value, sim.http_checkpoint), timeout_ms=30000, response_max_len=100)
            print("sim_task: HTTP response: "+repr(response))
            params.set_value("sim_status", "res")
            params.set_value("sim_successes", params.get("sim_successes").value + 1)
        except Exception as e:
            print("sim_task: Exception: "+repr(e))
            params.set_value("sim_status", "err")
            params.set_value("sim_failures", params.get("sim_failures").value + 1)
            
            if sim.http_checkpoint == 0:
                print("sim_task: Power cycling SIM7600")
                sim7600_power_on(False)
                await asyncio.sleep_ms(2000)
                sim7600_power_on(True)
        
        print("sim_task: successes: {}, failures: {}".format(
                params.get("sim_successes").value, params.get("sim_failures").value))

        params.set_value("http_checkpoint", sim.http_checkpoint)
        
        await asyncio.sleep_ms(5000)


asyncio.run(asyncio.gather(
    asyncio.create_task(backlight.update_task()),
    asyncio.create_task(display_task()),
    asyncio.create_task(event_task()),
    asyncio.create_task(adc_task()),
    asyncio.create_task(params_task()),
    asyncio.create_task(sim_task())
))


