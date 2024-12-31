print("UI8D v2.0")

import machine
from machine import Pin, SPI
import time
import utime
from pyb import Timer, ADC, UART
import sys
import ustruct
import math
from sim7600 import Sim

import portable_asyncio as asyncio
import backlight
import local_config

# Buttons
button_pins = [
    machine.Pin("BUTTON1", machine.Pin.IN, machine.Pin.PULL_UP),
    machine.Pin("BUTTON2", machine.Pin.IN, machine.Pin.PULL_UP),
    machine.Pin("BUTTON3", machine.Pin.IN, machine.Pin.PULL_UP),
    machine.Pin("BUTTON4", machine.Pin.IN, machine.Pin.PULL_UP),
    machine.Pin("BUTTON5", machine.Pin.IN, machine.Pin.PULL_UP),
]

# RS232_SHDN: RS232 to main board shutdown, active low
rs232_shdn = machine.Pin("PD10", machine.Pin.OUT)
rs232_shdn.high() # Wake up RS232 transceiver

# SIM7600 power control pin
sim7600_power_inhibit_pin = machine.Pin("PB9", machine.Pin.OUT)
def sim7600_power_on(on):
    if on:
        sim7600_power_inhibit_pin.low() # Active low
    else:
        sim7600_power_inhibit_pin.high()
sim7600_power_on(True)

# HVAC power-on output
hvac_power_on_pin = machine.Pin("PA15", machine.Pin.OUT) # PWMOUT1

# On-board NTC (10k pull-up, 10k NTC)
pcb_temperature_sensor_pin = Pin.board.PB1
pcb_temperature_sensor_adc = ADC(pcb_temperature_sensor_pin)
def read_pcb_temperature_c():
    adc_scaled = pcb_temperature_sensor_adc.read() / 4095.0
    if adc_scaled < 0.05:
        return None
    r_pull_up = 10000
    b_constant = 3977
    t0_k = 298.15
    r0 = 10000
    r_ntc = adc_scaled / (1.0 - adc_scaled) * r_pull_up
    t_k = 1.0 / ((1.0 / t0_k) + (1 / b_constant) * math.log(r_ntc / r0))
    t_c = t_k - 273.15
    return t_c

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

statusled[0] = (0, 1, 3)
statusled.write()

from ili9341 import Display, color565
# Yes it says 1 GHz. It's just picking the highest possible rate.
spi = SPI(3, baudrate=1000000000)
display = Display(spi, dc=Pin.board.PD14, cs=Pin.board.PD11, rst=Pin.board.PD13,
        width=320, height=240, rotation=90)

display.clear(color565(0, 0, 0))

from xglcd_font import XglcdFont
print('Loading fonts...')
font_unispace = XglcdFont('fonts/Unispace12x24.c', 12, 24)
print('...done.')

def draw_ui_text(self, x, y, text, color, align='L', force_pixel_length=None):
    text_x_off = 0
    if align == 'R':
        if force_pixel_length is not None:
            text_x_off = -force_pixel_length
        else:
            text_x_off = -font_unispace.measure_text(text)
    self.draw_text(x + text_x_off, y, text, font_unispace, color,
            force_pixel_length=force_pixel_length,
            right_align=(align == 'R'))
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
    def __init__(self, name, longname, unit, expect_min, expect_max, fmt,
            initial_value, canmap, reportmap):
        self.name = name
        self.longname = longname
        self.unit = unit
        self.expect_min = expect_min
        self.expect_max = expect_max
        self.fmt = fmt
        self.value = initial_value
        self.canmap = canmap
        self.reportmap = reportmap
        self.timeout = None

    def set_value(self, value, timeout=None):
        self.value = value
        self.timeout = timeout

class Params:
    def __init__(self):
        self.params = {}

    def add(self, name, longname, unit='', expect_min=None, expect_max=None, fmt="{:.2f}",
            initial_value=None, canmap=None, reportmap=None):
        self.params[name] = Param(name, longname, unit, expect_min, expect_max, fmt,
                initial_value, canmap, reportmap)

    def get(self, name):
        return self.params[name]

    def set_value(self, name, value, timeout=None):
        self.params[name].set_value(value, timeout)

    # Call once per second
    def update_timeouts(self):
        for param in self.params.values():
            if param.timeout is not None:
                param.timeout -= 1.0
                if param.timeout <= 0.0:
                    param.timeout = None
                    param.value = None

class CanMap:
    BIT = 0
    UINT8 = 1
    INT8 = 2
    FUNCTION = 3
    def __init__(self, id, type, pos, len=1, scale=1, timeout=10):
        self.id = id
        self.type = type
        self.pos = pos
        self.len = len
        self.scale = scale
        self.timeout = timeout
    def __str__(self):
        return "CanMap(0x{:x}, {}, {}, {}, {}, {})".format(self.id, self.type, self.pos,
                self.len, self.scale, self.timeout)

class ReportMap:
    def __init__(self, get_param, fmt="{:.0f}", scale=1):
        self.get_param = get_param
        self.fmt = fmt
        self.scale = scale
    def __str__(self):
        return "ReportMap({}, {}, {})".format(self.get_param, self.fmt, self.scale)

params = Params()

params.add("aux_voltage", "Aux voltage", "V", 0, 28,
        reportmap=ReportMap("vaux", "{:.1f}"))

params.add("ticks_ms", "ticks_ms", "", 1, 1000000, fmt="{:d}", initial_value=0,
        reportmap=ReportMap("t", "{:.0f}", 0.001))

params.add("sim_status", "SIM status")
params.add("sim_successes", "SIM successes", "", 1, 1000000, fmt="{:d}", initial_value=0)
params.add("sim_failures", "SIM failures", "", 0, 100, fmt="{:d}", initial_value=0)
params.add("http_checkpoint", "HTTP Checkp.", "", 10, 1000, fmt="{:d}", initial_value=0)
params.add("can_rx_count", "CAN RX #", "", 1, 1000000, fmt="{:d}", initial_value=0)
params.add("can_last_id", "CAN last id", "", 1, 0xfff, fmt="0x{:x}", initial_value=0)
params.add("hvac_countdown", "HVAC countdown", "s", 0, 10000, fmt="{:.0f}", initial_value=0)

def parse_outlander_heater_temperature(data):
    t1 = data[3] - 40
    t2 = data[3] - 40
    return t1 if t1 > t2 else t2
params.add("heater_t", "Heater T", "°C", -30, 100, fmt="{:.0f}",
        canmap=CanMap(0x398, CanMap.FUNCTION, parse_outlander_heater_temperature),
        reportmap=[ReportMap("ht", "{:d}"), ReportMap("oht", "{:d}")])

params.add("heater_heating", "Heater heating", "", 0, 1, fmt="{:d}",
        canmap=CanMap(0x398, CanMap.FUNCTION,
                lambda data: 1 if data[5] > 0 else 0),
        reportmap=[ReportMap("ohh", "{:d}")])

params.add("heater_power", "Heater power %", "", 0, 1, fmt="{:d}",
        canmap=CanMap(0x398, CanMap.FUNCTION,
                lambda data: 100 if data[5] > 0 else 0),
        reportmap=[ReportMap("he", "{:d}")])

params.add("soc", "SoC", "%", 0, 100, fmt="{:.1f}",
        canmap=CanMap(0x032, CanMap.UINT8, 6, scale=1/2.55, timeout=60),
        reportmap=ReportMap("er", "{:.0f}", 2.55))

params.add("cell_t_min", "Cell T min", "°C", -25, 65, fmt="{:.1f}",
        canmap=CanMap(0x031, CanMap.INT8, 3),
        reportmap=ReportMap("t0", "{:.0f}"))

params.add("cell_t_max", "Cell T max", "°C", -25, 65, fmt="{:.1f}",
        canmap=CanMap(0x031, CanMap.INT8, 4),
        reportmap=ReportMap("t1", "{:.0f}"))

params.add("cell_v_min", "Cell V min", "V", 2.95, 4.25,
        canmap=CanMap(0x031, CanMap.FUNCTION,
                lambda data: ((data[0] << 4) | (data[1] >> 4)) / 100.0),
        reportmap=ReportMap("v0", "{:.0f}", 100))

params.add("cell_v_max", "Cell V max", "V", 2.95, 4.25,
        canmap=CanMap(0x031, CanMap.FUNCTION,
                lambda data: (((data[1] & 0x0f) << 8) | data[2]) / 100.0),
        reportmap=ReportMap("v1", "{:.0f}", 100))

params.add("cabin_t", "Cabin T", "°C", -20, 60, fmt="{:.1f}",
        reportmap=ReportMap("cabin_t", "{:.1f}"))

params.add("main_contactor", "Main contactor", "", 0, 1, fmt="{:d}",
        canmap=CanMap(0x030, CanMap.BIT, 2), reportmap=ReportMap("mc", "{:d}"))

params.add("precharge_failed", "Prechg. failed", "", 0, 1, fmt="{:d}",
        canmap=CanMap(0x030, CanMap.BIT, 6), reportmap=ReportMap("pchg_f", "{:d}"))

params.add("balancing", "Balancing", "", 0, 1, fmt="{:d}",
        canmap=CanMap(0x031, CanMap.BIT, 5*8+0), reportmap=ReportMap("b", "{:d}"))

params.add("obc_dcv", "OBC DC V", "V", 0, 325, fmt="{:d}",
        canmap=CanMap(0x389, CanMap.UINT8, 0, scale=2),
        reportmap=ReportMap("pv", "{:d}", scale=10))

params.add("obc_dcc", "OBC DC A", "A", 0, 15, fmt="{:.1f}",
        canmap=CanMap(0x389, CanMap.UINT8, 2, scale=0.1),
        reportmap=ReportMap("pc", "{:.0f}", scale=10))

params.add("ac_voltage", "AC Voltage", "V", 0, 250, fmt="{:d}",
        canmap=CanMap(0x389, CanMap.UINT8, 1),
        reportmap=ReportMap("ac", "{:d}"))

def parse_pm_state(data):
    return data[5] & 0x0f
params.add("pm_state", "PM State", "", 0, 6, fmt="{:d}",
        canmap=CanMap(0x550, CanMap.FUNCTION, parse_pm_state),
        reportmap=ReportMap("pms", "{:d}"))

def parse_pm_cr(data):
    return (data[5] & 0xf0) >> 4
params.add("pm_cr", "PM Con Reas", "", 0, 3, fmt="{:d}",
        canmap=CanMap(0x550, CanMap.FUNCTION, parse_pm_cr),
        reportmap=ReportMap("pmcr", "{:d}"))

# Add a large number of parameters for finding out memory capacity
#for i in range(0,100):
#    params.add("dummy_"+str(i), "Dummy "+str(i), "?", 42, 1337,
#            canmap=CanMap(0x032, CanMap.UINT8, 6),
#            reportmap=ReportMap("dummy", "{:.0f}"))

canmap_by_id = {}
for param in params.params.values():
    if param.canmap is not None:
        canmap = param.canmap
        if canmap.id not in canmap_by_id:
            canmap_by_id[canmap.id] = [param]
        else:
            canmap_by_id[canmap.id].append(param)

views = {
    "main": {
        "params": [
            "ticks_ms",
            "aux_voltage",
            "sim_successes",
            "sim_failures",
            "http_checkpoint",
            "can_rx_count",
            "heater_t",
            "hvac_countdown",
        ],
    },
    "battery": {
        "params": [
            "soc",
            "cell_t_min",
            "cell_t_max",
            "cell_v_min",
            "cell_v_max",
            "cabin_t",
            "main_contactor",
            "precharge_failed",
        ],
    },
    "pm": {
        "params": [
            "pm_state",
            "pm_cr",
            "aux_voltage",
            "main_contactor",
            "precharge_failed",
        ],
    }
}

view_list = ["main", "battery", "pm"]
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
                display.draw_ui_text(250, 25 + i * 25, param.fmt.format(param.value),
                        color, align="R", force_pixel_length=60)
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
            display.clear(color565(0, 0, 0), hlines=2)
            display.draw_ui_text(319, 0, 'UI8D v2.0', color565(255, 0, 255), align='R')
            display.draw_ui_text(100, 0, "View "+str(current_view_index+1)+"/"+str(len(view_list)), color565(255, 0, 255))
            draw_view_labels(display, view)
        display.draw_ui_text(0, 0, str(i), color565(255, 0, 255))
        draw_view_values(display, view, drawn_values)
        display.update_simulator_view()
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
        
        # Vbat / auxV

        vbat_measure_avg = 0
        
        for i in range(0,10):
            vbat_measure_avg += vbat_measure_adc.read()

            await asyncio.sleep_ms(100)
        
        vbat_measure_avg /= 10
        
        params.set_value("aux_voltage", vbat_measure_avg / 4095.0 * 3.3 * 110.0 / 10.0, 5)

        # PCB temperature

        pcb_t = read_pcb_temperature_c()
        # Rough correction (will vary based on power saving modes and PCB revisions)
        cabin_t = pcb_t - 12
        params.set_value("cabin_t", cabin_t, 10)


async def params_task():
    while True:
        params.update_timeouts()
        
        params.set_value("ticks_ms", utime.ticks_ms())
        
        await asyncio.sleep_ms(1000)

# CAN

from pyb import CAN
#can = CAN(1, CAN.LOOPBACK)
can = CAN(1, CAN.NORMAL, baudrate=500_000)
# Accept anything (id, mask, id, mask)
can.setfilter(0, CAN.MASK16, 0, (0x000, 0x000, 0x000, 0x000))
#can.send('message!', 0x123)
#can.recv(0)

async def can_rx_task():
    while True:
        try:
            for i in range(0, 20):
                frame = can.recv(0, timeout=1)
                # $ cansend can0 555#deadbeef
                # -> Received CAN frame: (1365, False, False, 0, b'\xde\xad\xbe\xef')
                #print("Received CAN frame: "+repr(frame))
                params.set_value("can_rx_count", params.get("can_rx_count").value + 1)
                params.set_value("can_last_id", frame[0])

                id = frame[0]
                data = frame[4]

                try:
                    mapped_params = canmap_by_id[id]
                    for param in mapped_params:
                        canmap = param.canmap
                        if canmap.type == CanMap.BIT:
                            value = (canmap.scale
                                    if (data[canmap.pos // 8] & (1<<(canmap.pos % 8)))
                                    else 0)
                            param.set_value(value, canmap.timeout)
                        elif canmap.type == CanMap.UINT8:
                            value = data[canmap.pos] * canmap.scale
                            param.set_value(value, canmap.timeout)
                        elif canmap.type == CanMap.INT8:
                            value = (ustruct.unpack_from('>1b', data, canmap.pos)[0]
                                    * canmap.scale)
                            param.set_value(value, canmap.timeout)
                        elif canmap.type == CanMap.FUNCTION:
                            value = canmap.pos(data) * canmap.scale
                            param.set_value(value, canmap.timeout)
                        else:
                            print("canmap not supported: {}".format(param.canmap))
                except KeyError:
                    pass

        except OSError as e:
            # Probably timeout
            pass
        except Exception as e:
            print("can_rx_task: Exception: {}", str(e))
        await asyncio.sleep_ms(5)

#async def can_tx_task():
#    while True:
#        try:
#            can.send('message!', 0x123, timeout=100)
#        except OSError as e:
#            print("Failed to send CAN frame: "+str(e))
#        await asyncio.sleep_ms(500)

# HVAC power control

async def hvac_power_task():
    oscillation_prevention_counter = 0
    while True:
        countdown = params.get("hvac_countdown").value
        wanted_hvac_power_on = False
        if countdown > 0:
            countdown -= 0.5
            params.set_value("hvac_countdown", countdown)

            if params.get("aux_voltage").value >= 13.4:
                wanted_hvac_power_on = True

            # Request ipdm1 to turn on the heater and pump
            try:
                can.send('\x02\x00\x00\x00\x01\x00\x00\x00', 0x570, timeout=100)
            except OSError as e:
                pass # Probably timeout
        else:
            # Request ipdm1 to turn off the heater and pump
            try:
                can.send('\x02\x00\x00\x00\x00\x00\x00\x00', 0x570, timeout=100)
            except OSError as e:
                pass # Probably timeout
            
        if wanted_hvac_power_on:
            oscillation_prevention_counter += 1
            if oscillation_prevention_counter >= 10:
                if hvac_power_on_pin.value() == 0:
                    print("Setting HVAC power ON")
                hvac_power_on_pin.high()
        else:
            oscillation_prevention_counter = 0
            if hvac_power_on_pin.value() == 1:
                print("Setting HVAC power OFF")
            hvac_power_on_pin.low()

        await asyncio.sleep_ms(500)

# SIM7600

params.set_value("sim_status", "init")

print("SIM7600: UART")
sim_uart = UART(2, 115200, timeout=1000, read_buf_len=1000)
print("SIM7600: Sim")
sim = Sim(sim_uart)

def fmt_v(v, fmt, scale=1):
    if v is None:
        return "None"
    return fmt.format(v * scale)

async def sim_task():
    while True:
        try:
            request_url_parts = [local_config.GET_URL_PREFIX]
            for param in params.params.values():
                if param.reportmap is None:
                    pass
                elif type(param.reportmap) in (list, tuple):
                    for reportmap in param.reportmap:
                        if len(request_url_parts) >= 2:
                            request_url_parts.append("&")
                        request_url_parts.append(reportmap.get_param)
                        request_url_parts.append("=")
                        request_url_parts.append(fmt_v(param.value, reportmap.fmt, reportmap.scale))
                else:
                    reportmap = param.reportmap
                    if len(request_url_parts) >= 2:
                        request_url_parts.append("&")
                    request_url_parts.append(reportmap.get_param)
                    request_url_parts.append("=")
                    request_url_parts.append(fmt_v(param.value, reportmap.fmt, reportmap.scale))

            request_url = "".join(request_url_parts)

            print("sim_task: HTTP request: URL={}".format(request_url))
            params.set_value("sim_status", "req")
            response = await sim.http_get(request_url, timeout_ms=30000, response_max_len=100)
            print("sim_task: HTTP response: "+repr(response))
            params.set_value("sim_status", "res")
            params.set_value("sim_successes", params.get("sim_successes").value + 1)
            
            #if "404 Not Found" in response[2]:
            if "request_hvac_on" in response[2]:
                params.set_value("hvac_countdown", 60)

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

asyncio.run_until_complete(asyncio.gather(
    asyncio.create_task(backlight.update_task()),
    asyncio.create_task(display_task()),
    asyncio.create_task(event_task()),
    asyncio.create_task(adc_task()),
    asyncio.create_task(params_task()),
    asyncio.create_task(can_rx_task()),
    #asyncio.create_task(can_tx_task()),
    asyncio.create_task(hvac_power_task()),
    asyncio.create_task(sim_task())
))


