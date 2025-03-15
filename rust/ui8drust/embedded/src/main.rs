#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// Local modules

// Internal crates
use command_accumulator::CommandAccumulator;
use common::*;

// Platform-specific dependencies
use adc::{config::AdcConfig, Adc};
use critical_section::Mutex;
use hal::{
    adc::{self, config::SampleTime},
    gpio,
    gpio::PinExt,
    otg_fs, pac,
    prelude::*,
    serial,
};
use rtic_monotonics::{systick::*, Monotonic};
use stm32f4xx_hal as hal;
use usb_device::prelude::*;
use usbd_serial::{self, USB_CLASS_CDC};
// use eeprom24x::Eeprom24x;
use display_interface_spi::SPIInterface;
use embedded_hal_bus;
use ili9341::Ili9341;

// Standard library utilities
use core::{cell::RefCell, fmt::Write, ops::DerefMut};

// General purpose libraries
use arrayvec::{ArrayString, ArrayVec};
use embedded_graphics as eg;
use embedded_graphics::{
    draw_target::DrawTarget, mono_font, pixelcolor::*, prelude::Point, prelude::RgbColor,
    text::Text, Drawable,
};
use fixedstr::str_format;
use log::{debug, error, info, trace, warn, Log, Metadata, Record};
use micromath::F32Ext;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

// Constants

const LOG_BUFFER_SIZE: usize = 1024;
const CONSOLE_RX_BUF_SIZE: usize = 100;
const MAINBOARD_RX_BUF_SIZE: usize = 200;
const MAINBOARD_TX_BUF_SIZE: usize = 200;
const SIM7600_RX_BUF_SIZE: usize = 500;
const SIM7600_TX_BUF_SIZE: usize = 500;
const CAN_ENABLE_LOOPBACK_MODE: bool = false;

// Log buffering system

struct MultiLogger {
    uart_buffer: Mutex<RefCell<Option<ArrayString<LOG_BUFFER_SIZE>>>>,
    usb_buffer: Mutex<RefCell<Option<ArrayString<LOG_BUFFER_SIZE>>>>,
    display_buffer: Mutex<RefCell<Option<ArrayString<LOG_BUFFER_SIZE>>>>,
}

impl MultiLogger {
    fn get_uart_buffer(&self) -> Option<ArrayString<LOG_BUFFER_SIZE>> {
        let mut buf2: Option<ArrayString<LOG_BUFFER_SIZE>> = Some(ArrayString::new());
        critical_section::with(|cs| {
            // This replaces the logger buffer with an empty one, and we get the
            // possibly filled in one
            buf2 = self.uart_buffer.borrow(cs).replace(buf2);
        });
        buf2
    }
    fn get_usb_buffer(&self) -> Option<ArrayString<LOG_BUFFER_SIZE>> {
        let mut buf2: Option<ArrayString<LOG_BUFFER_SIZE>> = Some(ArrayString::new());
        critical_section::with(|cs| {
            // This replaces the logger buffer with an empty one, and we get the
            // possibly filled in one
            buf2 = self.usb_buffer.borrow(cs).replace(buf2);
        });
        buf2
    }
    fn get_display_buffer(&self) -> Option<ArrayString<LOG_BUFFER_SIZE>> {
        let mut buf2: Option<ArrayString<LOG_BUFFER_SIZE>> = Some(ArrayString::new());
        critical_section::with(|cs| {
            // This replaces the logger buffer with an empty one, and we get the
            // possibly filled in one
            buf2 = self.display_buffer.borrow(cs).replace(buf2);
        });
        buf2
    }
}

impl Log for MultiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Info // TODO: Adjust as needed
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            critical_section::with(|cs| {
                if let Some(ref mut buffer) = self.uart_buffer.borrow(cs).borrow_mut().deref_mut() {
                    let _ = buffer.write_fmt(format_args!(
                        "[{}] {}\r\n",
                        record.level(),
                        record.args()
                    ));
                    if buffer.is_full() {
                        let warning = " | LOG BUFFER FULL\r\n";
                        buffer.truncate(buffer.capacity() - warning.len());
                        let _ = buffer.try_push_str(warning);
                    }
                }
                if let Some(ref mut buffer) = self.usb_buffer.borrow(cs).borrow_mut().deref_mut() {
                    let _ = buffer.write_fmt(format_args!(
                        "[{}] {}\r\n",
                        record.level(),
                        record.args()
                    ));
                    if buffer.is_full() {
                        let warning = " | LOG BUFFER FULL\r\n";
                        buffer.truncate(buffer.capacity() - warning.len());
                        let _ = buffer.try_push_str(warning);
                    }
                }
                if let Some(ref mut buffer) =
                    self.display_buffer.borrow(cs).borrow_mut().deref_mut()
                {
                    let _ = buffer.write_fmt(format_args!("{}\r\n", record.args()));
                    if buffer.is_full() {
                        let warning = " | LOG BUFFER FULL\r\n";
                        buffer.truncate(buffer.capacity() - warning.len());
                        let _ = buffer.try_push_str(warning);
                    }
                }
            });
            // Trigger write to hardware by triggering USART1 interrupt
            pac::NVIC::pend(pac::Interrupt::USART1);
            // Trigger write to hardware by triggering OTG_FS interrupt
            pac::NVIC::pend(pac::Interrupt::OTG_FS);
        }
    }

    fn flush(&self) {
        // Flushing is handled elsewhere
    }
}

static MULTI_LOGGER: MultiLogger = MultiLogger {
    uart_buffer: Mutex::new(RefCell::new(None)),
    usb_buffer: Mutex::new(RefCell::new(None)),
    display_buffer: Mutex::new(RefCell::new(None)),
};

// Function to initialize the logger
fn init_logger() {
    critical_section::with(|cs| {
        MULTI_LOGGER
            .uart_buffer
            .borrow(cs)
            .replace(Some(ArrayString::new()));
        MULTI_LOGGER
            .usb_buffer
            .borrow(cs)
            .replace(Some(ArrayString::new()));
        MULTI_LOGGER
            .display_buffer
            .borrow(cs)
            .replace(Some(ArrayString::new()));
    });
    log::set_logger(&MULTI_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info); // TODO: Adjust as needed
}

// CAN driver

pub struct CAN1 {
    _private: (),
}
unsafe impl bxcan::Instance for CAN1 {
    const REGISTERS: *mut bxcan::RegisterBlock = 0x4000_6400 as *mut _;
}
unsafe impl bxcan::FilterOwner for CAN1 {
    const NUM_FILTER_BANKS: u8 = 28;
}

// TIM4 PWM: LCD backlight PWM and PWMOUT2

type Tim4Pwm = hal::timer::PwmHz<
    hal::pac::TIM4,
    (
        hal::timer::ChannelBuilder<hal::pac::TIM4, 0, false>,
        hal::timer::ChannelBuilder<hal::pac::TIM4, 3, false>,
    ),
>;

fn set_lcd_backlight(pwm: f32, pwm_timer: &mut Tim4Pwm) {
    pwm_timer.set_duty(
        hal::timer::Channel::C1,
        (pwm_timer.get_max_duty() as f32 * pwm) as u16,
    );
}

fn set_pwmout2(pwm: f32, pwm_timer: &mut Tim4Pwm) {
    pwm_timer.set_duty(
        hal::timer::Channel::C4,
        (pwm_timer.get_max_duty() as f32 * pwm) as u16,
    );
}

// Buttons

type Button1Pin = gpio::Pin<'A', 4, gpio::Input>;
type Button2Pin = gpio::Pin<'E', 1, gpio::Input>;
type Button3Pin = gpio::Pin<'E', 2, gpio::Input>;
type Button4Pin = gpio::Pin<'E', 3, gpio::Input>;
type Button5Pin = gpio::Pin<'E', 4, gpio::Input>;
type WkupPin = gpio::Pin<'A', 0, gpio::Input>;

// HardwareInterface implementation

type Display = Ili9341<
    SPIInterface<
        embedded_hal_bus::spi::CriticalSectionDevice<
            'static,
            stm32f4xx_hal::spi::Spi<pac::SPI3>,
            gpio::PD11<gpio::Output<gpio::PushPull>>,
            embedded_hal_bus::spi::NoDelay,
        >,
        gpio::PD14<gpio::Output<gpio::PushPull>>,
    >,
    gpio::PD13<gpio::Output<gpio::PushPull>>,
>;

type Boot0ControlPin = gpio::Pin<'A', 8, gpio::Output<gpio::PushPull>>;
type WakeupOutputPin = gpio::Pin<'E', 0, gpio::Output<gpio::PushPull>>;
// TODO: Change this into an actual PWM output
type Pwmout1Pin = gpio::Pin<'A', 15, gpio::Output<gpio::PushPull>>;
type Sim7600PowerInhibitPin = gpio::PB9<gpio::Output<gpio::PushPull>>;

struct HardwareImplementation {
    display: &'static mut Display,
    boot0_control_pin: &'static mut Boot0ControlPin,
    wakeup_output_pin: WakeupOutputPin,
    pwmout1_pin: Pwmout1Pin,
    sim7600_power_inhibit_pin: Sim7600PowerInhibitPin,
    sim7600driver: Sim7600Driver,
    can_tx_buf: ConstGenericRingBuffer<bxcan::Frame, 10>,
    adc_result_vbat: f32,
}

impl HardwareInterface for HardwareImplementation {
    fn millis(&mut self) -> u64 {
        // NOTE: This rolls over at 49.71 days
        Systick::now().duration_since_epoch().to_millis() as u64
    }

    fn display_clear(&mut self, color: Rgb565) {
        self.display.clear(color).unwrap();
    }

    fn display_draw_text(
        &mut self,
        text: &str,
        p: Point,
        style: mono_font::MonoTextStyle<Rgb565>,
        alignment: eg::text::Alignment,
    ) {
        Text::with_alignment(text, p, style, alignment)
            .draw(self.display)
            .unwrap();
    }

    fn activate_dfu(&mut self) {
        self.boot0_control_pin.set_high();
        long_busywait();
        cortex_m::peripheral::SCB::sys_reset();
    }

    fn http_get_start(&mut self, url: &str) {
        info!("http_get_start(): url: {:?}", url);

        self.sim7600driver.http_get_start(url);
    }

    fn http_get_update(&mut self) -> HttpUpdateStatus {
        self.sim7600driver.http_get_update()
    }

    fn http_get_stop(&mut self) {
        self.sim7600driver.http_get_stop()
    }

    fn send_can(&mut self, frame: bxcan::Frame) {
        //info!("send_can(): {:?}", frame);
        self.can_tx_buf.push(frame);
    }

    fn get_analog_input(&mut self, input: AnalogInput) -> f32 {
        match input {
            AnalogInput::AuxVoltage => self.adc_result_vbat,
            _ => f32::NAN,
        }
    }

    fn set_digital_output(&mut self, output: DigitalOutput, value: bool) {
        // TODO
        match output {
            DigitalOutput::Wakeup => { self.wakeup_output_pin.set_state(value.into()) }
            DigitalOutput::Pwmout1 => { self.pwmout1_pin.set_state(value.into()) }
            DigitalOutput::Sim7600PowerInhibit => { self.sim7600_power_inhibit_pin.set_state(value.into()) }
        }
    }
}

// Panic output and input methods

static mut PANIC_TX: Option<hal::serial::Tx<hal::pac::USART1, u8>> = None;
static mut PANIC_DISPLAY: Option<Display> = None;
static mut PANIC_BOOT0_CONTROL_PIN: Option<Boot0ControlPin> = None;
static mut PANIC_BUTTON1_PIN: Option<Button1Pin> = None;
static mut PANIC_BUTTON3_PIN: Option<Button3Pin> = None;

// RTIC application

#[rtic::app(device = hal::pac, peripherals = true, dispatchers = [UART5, I2C3_ER, USART6])]
mod rtic_app {
    use super::*;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, otg_fs::UsbBusType>,
        usb_serial: usbd_serial::SerialPort<'static, otg_fs::UsbBusType>,
        console_rxbuf: ConstGenericRingBuffer<u8, CONSOLE_RX_BUF_SIZE>,
        sim7600_rxbuf: ConstGenericRingBuffer<u8, SIM7600_RX_BUF_SIZE>,
        sim7600_txbuf: ConstGenericRingBuffer<u8, SIM7600_TX_BUF_SIZE>,
        mainboard_rxbuf: ConstGenericRingBuffer<u8, MAINBOARD_RX_BUF_SIZE>,
        mainboard_txbuf: ConstGenericRingBuffer<u8, MAINBOARD_TX_BUF_SIZE>,
        can1: bxcan::Can<CAN1>,
        can_rx_buf: ConstGenericRingBuffer<bxcan::Frame, 10>,
        can_tx_buf: ConstGenericRingBuffer<bxcan::Frame, 10>,
        button1_pin: &'static mut Button1Pin,
        button2_pin: Button2Pin,
        button3_pin: &'static mut Button3Pin,
        button4_pin: Button4Pin,
        button5_pin: Button5Pin,
        wkup_pin: WkupPin, // The WKUP input pin
        button_event_queue: ConstGenericRingBuffer<ButtonEvent, 10>,
        adc_result_ldr: u16,
        adc_result_vbat: f32,
    }

    #[local]
    struct Local {
        usart1_rx: hal::serial::Rx<hal::pac::USART1, u8>,
        usart1_tx: &'static mut hal::serial::Tx<hal::pac::USART1, u8>,
        usart2_rx: hal::serial::Rx<hal::pac::USART2, u8>,
        usart2_tx: hal::serial::Tx<hal::pac::USART2, u8>,
        //usart3_rx: hal::serial::Rx<hal::pac::USART3, u8>,
        //usart3_tx: hal::serial::Tx<hal::pac::USART3, u8>,
        command_accumulator: CommandAccumulator<50>,
        i2c1: hal::i2c::I2c<hal::pac::I2C1>,
        adc1: Adc<pac::ADC1>,
        // Analog input pins
        adc_pa1: gpio::Pin<'A', 1, gpio::Analog>,
        adc_pa2: gpio::Pin<'A', 2, gpio::Analog>,
        adc_pa3: gpio::Pin<'A', 3, gpio::Analog>,
        // Digital input pins
        // Output pins
        // (See HardwareImplementation)
        // PWM output timers
        tim4_pwm: Tim4Pwm,
        last_backlight_pwm: f32,
        // Other
        hw: HardwareImplementation,
    }

    #[init()]
    fn init(mut cx: init::Context) -> (Shared, Local) {
        static mut EP_MEMORY: [u32; 1024] = [0; 1024];
        static mut USB_BUS: Option<usb_device::bus::UsbBusAllocator<otg_fs::UsbBusType>> = None;
        static mut SPI3_SHARED: Option<Mutex<RefCell<hal::spi::Spi<hal::pac::SPI3>>>> = None;

        // System clock

        // Enable CAN1
        cx.device.RCC.apb1enr.modify(|_, w| w.can1en().enabled());

        let rcc = cx.device.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(8.MHz()) // Use external crystal (HSE)
            .hclk(168.MHz())
            .pclk1(42.MHz())
            .pclk2(84.MHz())
            .sysclk(168.MHz()) // Set system clock (SYSCLK)
            .freeze(); // Apply the configuration

        let mut syscfg = cx.device.SYSCFG.constrain();

        // Pin assignments

        let gpioa = cx.device.GPIOA.split();
        let gpiob = cx.device.GPIOB.split();
        let gpioc = cx.device.GPIOC.split();
        let gpiod = cx.device.GPIOD.split();
        let gpioe = cx.device.GPIOE.split();

        // Output pins

        let sim7600_power_inhibit_pin = gpiob
            .pb9
            .into_push_pull_output_in_state(gpio::PinState::Low);

        let boot0_control_pin = gpioa.pa8.into_push_pull_output();
        let boot0_control_pin = unsafe {
            PANIC_BOOT0_CONTROL_PIN = Some(boot0_control_pin);
            PANIC_BOOT0_CONTROL_PIN.as_mut().unwrap()
        };

        let mut wakeup_output_pin = gpioe.pe0.into_push_pull_output();
        let mut pwmout1_pin = gpioa.pa15.into_push_pull_output();

        // External interrupt pins

        // NOTE: button1 and button5 both use EXTI4. On stm32f4, it's not
        // possible to have two pins be the source of the same external
        // interrupt. Thus, we use the PA0(WKUP) pin to detect button1 presses,
        // as it is wired to go high every time a button is pressed and it can
        // act as EXTI0 source.
        let mut button1_pin = gpioa.pa4.into_pull_up_input();
        //button1_pin.make_interrupt_source(&mut syscfg);
        //button1_pin.enable_interrupt(&mut cx.device.EXTI);
        //button1_pin.trigger_on_edge(&mut cx.device.EXTI, gpio::Edge::Falling);
        let button1_pin = unsafe {
            PANIC_BUTTON1_PIN = Some(button1_pin);
            PANIC_BUTTON1_PIN.as_mut().unwrap()
        };
        let mut wkup_pin = gpioa.pa0.into_input();
        wkup_pin.make_interrupt_source(&mut syscfg);
        wkup_pin.enable_interrupt(&mut cx.device.EXTI);
        wkup_pin.trigger_on_edge(&mut cx.device.EXTI, gpio::Edge::RisingFalling);

        let mut button2_pin = gpioe.pe1.into_pull_up_input();
        button2_pin.make_interrupt_source(&mut syscfg);
        button2_pin.enable_interrupt(&mut cx.device.EXTI);
        button2_pin.trigger_on_edge(&mut cx.device.EXTI, gpio::Edge::Falling);

        let mut button3_pin = gpioe.pe2.into_pull_up_input();
        let button3_pin = unsafe {
            PANIC_BUTTON3_PIN = Some(button3_pin);
            PANIC_BUTTON3_PIN.as_mut().unwrap()
        };
        button3_pin.make_interrupt_source(&mut syscfg);
        button3_pin.enable_interrupt(&mut cx.device.EXTI);
        button3_pin.trigger_on_edge(&mut cx.device.EXTI, gpio::Edge::Falling);

        let mut button4_pin = gpioe.pe3.into_pull_up_input();
        button4_pin.make_interrupt_source(&mut syscfg);
        button4_pin.enable_interrupt(&mut cx.device.EXTI);
        button4_pin.trigger_on_edge(&mut cx.device.EXTI, gpio::Edge::Falling);

        let mut button5_pin = gpioe.pe4.into_pull_up_input();
        button5_pin.make_interrupt_source(&mut syscfg);
        button5_pin.enable_interrupt(&mut cx.device.EXTI);
        button5_pin.trigger_on_edge(&mut cx.device.EXTI, gpio::Edge::Falling);

        // SysTick

        let systick_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 168_000_000, systick_token); // Eats SYST peripheral

        // Software utilities

        init_logger();

        info!("-!- ui8d boot");

        // SPI bus

        // The SPI bus is shared between the SPI LCD and the optional LoRa and
        // W5500 modules

        // The SPI bus is accessed from different interrupts, so we have to use
        // embedded_hal_bus::spi::CriticalSectionDevice. If it was accessed from
        // only a single interrupt, we could use RefCellDevice instead.

        let spi3 = hal::spi::Spi::new(
            cx.device.SPI3,
            (
                gpiob.pb3.into_alternate::<6>(),
                gpiob.pb4.into_alternate::<6>(),
                gpiob.pb5.into_alternate::<6>().internal_pull_up(true),
            ),
            hal::spi::Mode {
                polarity: hal::spi::Polarity::IdleLow,
                phase: hal::spi::Phase::CaptureOnFirstTransition,
            },
            40.MHz(),
            &clocks,
        );

        let spi3_shared = unsafe {
            // The SPI3 bus shared by multiple peripherals is stored here. The
            // static lifetime makes things a lot easier
            SPI3_SHARED = Some(Mutex::new(RefCell::new(spi3)));
            SPI3_SHARED.as_mut().unwrap()
        };

        // LCD backlight and PWMOUT2

        let backlight_ch: hal::timer::ChannelBuilder<hal::pac::TIM4, 0, false> =
            hal::timer::Channel1::new(gpiod.pd12);
        let pwmout_ch: hal::timer::ChannelBuilder<hal::pac::TIM4, 3, false> =
            hal::timer::Channel4::new(gpiod.pd15);

        let mut tim4_pwm = cx
            .device
            .TIM4
            .pwm_hz((backlight_ch, pwmout_ch), 1000.Hz(), &clocks);

        tim4_pwm.enable(hal::timer::Channel::C1);
        tim4_pwm.enable(hal::timer::Channel::C4);
        set_lcd_backlight(0.2, &mut tim4_pwm);
        set_pwmout2(0.20, &mut tim4_pwm);

        // SPI LCD

        let mut lcd_reset_pin = gpiod.pd13.into_push_pull_output();
        let mut lcd_cs_pin = gpiod.pd11.into_push_pull_output();
        let lcd_dc_pin = gpiod.pd14.into_push_pull_output();

        let spi3_lcd =
            embedded_hal_bus::spi::CriticalSectionDevice::new_no_delay(spi3_shared, lcd_cs_pin);

        let spi_iface = SPIInterface::new(spi3_lcd, lcd_dc_pin);

        let mut delay = cx.device.TIM1.delay_us(&clocks);
        let mut display = Ili9341::new(
            spi_iface,
            lcd_reset_pin,
            &mut delay,
            ili9341::Orientation::LandscapeFlipped,
            ili9341::DisplaySize240x320,
        )
        .unwrap();

        let display = unsafe {
            PANIC_DISPLAY = Some(display);
            PANIC_DISPLAY.as_mut().unwrap()
        };

        display.clear(Rgb565::BLACK).unwrap();

        let style =
            mono_font::MonoTextStyle::new(&mono_font::iso_8859_10::FONT_10X20, Rgb565::WHITE);
        Text::with_alignment(
            "ui8d",
            Point::new(319, 24),
            style,
            eg::text::Alignment::Right,
        )
        .draw(display)
        .unwrap();

        // USART1 (TX=PA9, RX=PA10): TTL serial on programming header. We
        // provide our serial console here, and also on native USB. Note that
        // PA9 is also USB VBUS detect, because the bootloader wants that there.

        let serial_usart1: serial::Serial<pac::USART1, u8> = serial::Serial::new(
            cx.device.USART1,
            (
                gpioa.pa9.into_alternate::<7>(),
                gpioa.pa10.into_alternate::<7>(),
            ),
            serial::config::Config::default().baudrate(19200.bps()),
            &clocks,
        )
        .unwrap();
        let (usart1_tx, mut usart1_rx) = serial_usart1.split();
        usart1_rx.listen();
        let usart1_tx = unsafe {
            PANIC_TX = Some(usart1_tx);
            PANIC_TX.as_mut().unwrap()
        };

        // USART2 (TX=PD5, RX=PD6): SIM7600 UART

        let serial_usart2: serial::Serial<pac::USART2, u8> = serial::Serial::new(
            cx.device.USART2,
            (
                gpiod.pd5.into_alternate::<7>(),
                gpiod.pd6.into_alternate::<7>(),
            ),
            serial::config::Config::default().baudrate(115200.bps()),
            &clocks,
        )
        .unwrap();
        let (usart2_tx, mut usart2_rx) = serial_usart2.split();
        usart2_rx.listen();

        // USART3 (TX=PD8, RX=PD9): RS232 to iPDM56v2. These pins should be
        // floated when direct communication to mainboard via USB connector is
        // wanted, e.g.  when flashing the main board or when using the
        // mainboard serial console via USB.
        // TODO: It's strictly disabled now to make sure the pins are floating.

        /*let serial_usart3: serial::Serial<pac::USART3, u8> = serial::Serial::new(
            cx.device.USART3,
            (gpiod.pd8.into_alternate::<7>(),
            gpiod.pd9.internal_pull_up(true).into_alternate::<7>()),
            serial::config::Config::default().baudrate(115200.bps()),
            &clocks
        ).unwrap();
        let (usart3_tx, mut usart3_rx) = serial_usart3.split();
        usart3_rx.listen();*/

        // USB

        let usb = otg_fs::USB::new(
            (
                cx.device.OTG_FS_GLOBAL,
                cx.device.OTG_FS_DEVICE,
                cx.device.OTG_FS_PWRCLK,
            ),
            (
                gpioa.pa11.into_alternate::<10>(),
                gpioa.pa12.into_alternate::<10>(),
            ),
            &clocks,
        );

        unsafe {
            USB_BUS.replace(otg_fs::UsbBus::new(usb, &mut EP_MEMORY));
        }

        let usb_serial = usbd_serial::SerialPort::new(unsafe { USB_BUS.as_ref().unwrap() });

        let usb_dev = UsbDeviceBuilder::new(
            unsafe { USB_BUS.as_ref().unwrap() },
            //UsbVidPid(0x1209, 0x0001)) // https://pid.codes/1209/0001/
            UsbVidPid(0x0483, 0x5740),
        ) // STMicroelectronics / Virtual COM Port
        .device_class(USB_CLASS_CDC)
        .strings(&[
            StringDescriptors::new(usb_device::descriptor::lang_id::LangID::EN)
                .manufacturer("8Dromeda Productions")
                .product("ui8d")
                .serial_number("1337"),
        ])
        .unwrap()
        .build();

        // ADC

        let adc_pa1 = gpioa.pa1.into_analog(); // Version detection
        let adc_pa2 = gpioa.pa2.into_analog(); // Vbat
        let adc_pa3 = gpioa.pa3.into_analog(); // LDR
                                               // TODO: More pins

        let adc_config = AdcConfig::default()
            .resolution(adc::config::Resolution::Twelve)
            .clock(adc::config::Clock::Pclk2_div_8);

        let adc1 = Adc::adc1(cx.device.ADC1, true, adc_config);

        // I2C
        // There's a 24C02 EEPROM chip on this bus

        let mut i2c1 = hal::i2c::I2c::new(
            cx.device.I2C1,
            (gpiob.pb8, gpiob.pb7),
            hal::i2c::Mode::Standard {
                frequency: 400.kHz(),
            },
            &clocks,
        );

        // CAN

        let _pins = (
            gpiod.pd1.into_alternate::<9>(), // CAN1 TX
            gpiod.pd0.into_alternate::<9>(), // CAN1 RX
        );

        let mut can1 = bxcan::Can::builder(CAN1 { _private: () })
            .set_loopback(CAN_ENABLE_LOOPBACK_MODE)
            .set_bit_timing(0x00090006) // 500kbps at 42MHz pclk1
            .enable();

        can1.modify_filters()
            .enable_bank(0, bxcan::Fifo::Fifo0, bxcan::filter::Mask32::accept_all())
            .enable_bank(1, bxcan::Fifo::Fifo1, bxcan::filter::Mask32::accept_all());

        can1.enable_interrupt(bxcan::Interrupt::Fifo0MessagePending);
        can1.enable_interrupt(bxcan::Interrupt::Fifo1MessagePending);
        can1.enable_interrupt(bxcan::Interrupt::TransmitMailboxEmpty);

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::CAN1_RX0);
            pac::NVIC::unmask(pac::Interrupt::CAN1_RX1);
            pac::NVIC::unmask(pac::Interrupt::CAN1_TX);
            pac::NVIC::unmask(pac::Interrupt::CAN1_SCE);
        }

        // Hardware abstraction

        let hw = HardwareImplementation {
            display: display,
            boot0_control_pin,
            wakeup_output_pin,
            pwmout1_pin,
            sim7600_power_inhibit_pin: sim7600_power_inhibit_pin,
            sim7600driver: Sim7600Driver::new(),
            can_tx_buf: ConstGenericRingBuffer::new(),
            adc_result_vbat: f32::NAN,
        };

        // Schedule tasks

        ui_task::spawn().ok();
        adc_task::spawn().ok();

        // Initialize context

        (
            Shared {
                console_rxbuf: ConstGenericRingBuffer::new(),
                sim7600_rxbuf: ConstGenericRingBuffer::new(),
                sim7600_txbuf: ConstGenericRingBuffer::new(),
                mainboard_rxbuf: ConstGenericRingBuffer::new(),
                mainboard_txbuf: ConstGenericRingBuffer::new(),
                usb_dev: usb_dev,
                usb_serial: usb_serial,
                can1: can1,
                can_rx_buf: ConstGenericRingBuffer::new(),
                can_tx_buf: ConstGenericRingBuffer::new(),
                button1_pin: button1_pin,
                button2_pin: button2_pin,
                button3_pin: button3_pin,
                button4_pin: button4_pin,
                button5_pin: button5_pin,
                wkup_pin: wkup_pin,
                button_event_queue: ConstGenericRingBuffer::new(),
                adc_result_ldr: 0,
                adc_result_vbat: 0.0,
            },
            Local {
                usart1_rx: usart1_rx,
                usart1_tx: usart1_tx,
                usart2_rx: usart2_rx,
                usart2_tx: usart2_tx,
                //usart3_rx: usart3_rx,
                //usart3_tx: usart3_tx,
                command_accumulator: CommandAccumulator::new(),
                i2c1: i2c1,
                adc1: adc1,
                adc_pa1: adc_pa1,
                adc_pa2: adc_pa2,
                adc_pa3: adc_pa3,
                tim4_pwm: tim4_pwm,
                last_backlight_pwm: 0.2,
                hw,
            },
        )
    }

    #[idle(
        shared = [
        ]
    )]
    fn idle(mut cx: idle::Context) -> ! {
        loop {
            short_busywait();
            //cx.shared.debug_pin.lock(|pin| { pin.toggle(); });
        }
    }

    #[task(priority = 1,
        shared = [
            console_rxbuf,
            sim7600_rxbuf,
            sim7600_txbuf,
            mainboard_rxbuf,
            mainboard_txbuf,
            can1,
            can_rx_buf,
            can_tx_buf,
            button_event_queue,
            adc_result_ldr,
            adc_result_vbat,
        ],
        local = [
            command_accumulator,
            hw,
            tim4_pwm,
            last_backlight_pwm,
        ]
    )]
    async fn ui_task(mut cx: ui_task::Context) {
        let mut state = app::MainState::new();

        loop {
            // Update values
            cx.local.hw.adc_result_vbat = cx.shared.adc_result_vbat.lock(|v| *v);

            // Set backlight PWM based on LDR brightness measurement
            let adc_result_ldr = cx.shared.adc_result_ldr.lock(|v| *v);
            let ldr_percent = adc_result_ldr as f32 / 4095.0 * 100.0;
            let wanted_backlight_pwm = (ldr_percent / 100.0).max(0.01);
            // Lowpass filter
            let new_backlight_pwm =
                wanted_backlight_pwm * 0.01 + *cx.local.last_backlight_pwm * 0.99;
            *cx.local.last_backlight_pwm = new_backlight_pwm;
            set_lcd_backlight(new_backlight_pwm, &mut cx.local.tim4_pwm);

            // Handle button events
            while let Some(event) = cx.shared.button_event_queue.lock(|queue| queue.dequeue()) {
                state.on_button_event(event, cx.local.hw);
            }

            state.update(cx.local.hw);

            // Handle CAN receive buffer
            while let Some(received_frame) =
                cx.shared.can_rx_buf.lock(|can_rx_buf| can_rx_buf.dequeue())
            {
                state.on_can(received_frame);
            }
            // Handle CAN transmit buffer
            while let Some(frame) = cx.local.hw.can_tx_buf.dequeue() {
                cx.shared
                    .can_tx_buf
                    .lock(|can_tx_buf| can_tx_buf.push(frame));
            }

            // Handle SIM7600 driver buffers
            let millis = cx.local.hw.millis();
            cx.local.hw.sim7600driver.update_time(millis);
            while let Some(b) = cx.local.hw.sim7600driver.buffers.txbuf.dequeue() {
                cx.shared.sim7600_txbuf.lock(|buf| buf.push(b));
                // Trigger write to hardware by triggering USART2 interrupt
                pac::NVIC::pend(pac::Interrupt::USART2);
            }
            while let Some(b) = cx.shared.sim7600_rxbuf.lock(|rxbuf| rxbuf.dequeue()) {
                cx.local.hw.sim7600driver.push(b);
            }

            // Handle console commands
            while let Some(b) = cx.shared.console_rxbuf.lock(|rxbuf| rxbuf.dequeue()) {
                if let Some(command) = cx.local.command_accumulator.put(b as char) {
                    info!("Command: {:?}", command);
                    if state.on_console_command(&command, cx.local.hw) {
                        // Higher level logic handled the command
                    } else {
                        info!(
                            "-> {:?} is an unknown command. Available commands:",
                            command
                        );
                        state.list_console_commands();
                    }
                }
            }

            // Handle log display buffer
            let logger_display_buf_option = MULTI_LOGGER.get_display_buffer();
            if let Some(logger_display_buf) = logger_display_buf_option {
                state.store_log_for_display(&logger_display_buf);
            }

            Systick::delay(15.millis()).await;
        }
    }

    #[task(priority = 2,
        shared = [
            adc_result_ldr,
            adc_result_vbat,
        ],
        local = [
            adc1,
            adc_pa1,
            adc_pa2,
            adc_pa3,
        ]
    )]
    async fn adc_task(mut cx: adc_task::Context) {
        let mut mux_channel: usize = 0;
        loop {
            // NOTE: DMA seemed to work, until it stopped after an essentially
            // random amount of time. Thus, we are doing it this way.

            //let adc_result_hwver = cx.local.adc.convert(cx.local.adc_pa1, SampleTime::Cycles_480);

            let adc_result_vbat =
                cx.local
                    .adc1
                    .convert(cx.local.adc_pa2, SampleTime::Cycles_480) as f32
                    * 0.00881;

            // Assign with lowpass
            cx.shared.adc_result_vbat.lock(|v| *v = *v * 0.98 + adc_result_vbat * 0.02);

            let adc_result_ldr = cx
                .local
                .adc1
                .convert(cx.local.adc_pa3, SampleTime::Cycles_480);

            cx.shared.adc_result_ldr.lock(|v| *v = adc_result_ldr);

            Systick::delay(20.millis()).await;
        }
    }

    // External interrupts for buttons

    #[task(
        binds = EXTI0,
        shared = [
            wkup_pin,
            button1_pin,
            button_event_queue,
        ],
        local = [
            button1_last_pressed: bool = false,
        ]
    )]
    fn exti0(mut cx: exti0::Context) {
        cx.shared
            .wkup_pin
            .lock(|pin| pin.clear_interrupt_pending_bit());
        let button1_pressed = cx.shared.button1_pin.lock(|pin| pin.is_low());
        //info!("EXTI0 (WKUP)");
        // button1 presses are detected here (as it can't have its own
        // interrupt)
        if button1_pressed && !*cx.local.button1_last_pressed {
            info!("WKUP: button1 down event");
            cx.shared.button_event_queue.lock(|button_event_queue| {
                button_event_queue.push(ButtonEvent::ButtonPress(Button::Button1));
            });
        }
        *cx.local.button1_last_pressed = button1_pressed;
    }

    #[task(
        binds = EXTI1,
        shared = [
            button2_pin,
            button_event_queue,
        ],
        local = [
            last_pressed_timestamp: u32 = 0,
        ]
    )]
    fn exti1(mut cx: exti1::Context) {
        cx.shared
            .button2_pin
            .lock(|pin| pin.clear_interrupt_pending_bit());
        // Debouncing
        let millis = Systick::now().duration_since_epoch().to_millis();
        if millis > *cx.local.last_pressed_timestamp
            && millis - *cx.local.last_pressed_timestamp > 50
        {
            *cx.local.last_pressed_timestamp = millis;
            info!("EXTI1: button2 down event");
            cx.shared.button_event_queue.lock(|button_event_queue| {
                button_event_queue.push(ButtonEvent::ButtonPress(Button::Button2));
            });
        }
    }

    #[task(
        binds = EXTI2,
        shared = [
            button3_pin,
            button_event_queue,
        ],
        local = [
            last_pressed_timestamp: u32 = 0,
        ]
    )]
    fn exti2(mut cx: exti2::Context) {
        cx.shared
            .button3_pin
            .lock(|pin| pin.clear_interrupt_pending_bit());
        // Debouncing
        let millis = Systick::now().duration_since_epoch().to_millis();
        if millis > *cx.local.last_pressed_timestamp
            && millis - *cx.local.last_pressed_timestamp > 50
        {
            *cx.local.last_pressed_timestamp = millis;
            info!("EXTI2: button3 down event");
            cx.shared.button_event_queue.lock(|button_event_queue| {
                button_event_queue.push(ButtonEvent::ButtonPress(Button::Button3));
            });
        }
    }

    #[task(
        binds = EXTI3,
        shared = [
            button4_pin,
            button_event_queue,
        ],
        local = [
            last_pressed_timestamp: u32 = 0,
        ]
    )]
    fn exti3(mut cx: exti3::Context) {
        cx.shared
            .button4_pin
            .lock(|pin| pin.clear_interrupt_pending_bit());
        // Debouncing
        let millis = Systick::now().duration_since_epoch().to_millis();
        if millis > *cx.local.last_pressed_timestamp
            && millis - *cx.local.last_pressed_timestamp > 50
        {
            *cx.local.last_pressed_timestamp = millis;
            info!("EXTI3: button4 down event");
            cx.shared.button_event_queue.lock(|button_event_queue| {
                button_event_queue.push(ButtonEvent::ButtonPress(Button::Button4));
            });
        }
    }

    #[task(
        binds = EXTI4,
        shared = [
            button5_pin,
            button_event_queue,
        ],
        local = [
            last_pressed_timestamp: u32 = 0,
        ]
    )]
    fn exti4(mut cx: exti4::Context) {
        cx.shared
            .button5_pin
            .lock(|pin| pin.clear_interrupt_pending_bit());
        // Debouncing
        let millis = Systick::now().duration_since_epoch().to_millis();
        if millis > *cx.local.last_pressed_timestamp
            && millis - *cx.local.last_pressed_timestamp > 50
        {
            *cx.local.last_pressed_timestamp = millis;
            info!("EXTI4: button5 down event");
            cx.shared.button_event_queue.lock(|button_event_queue| {
                button_event_queue.push(ButtonEvent::ButtonPress(Button::Button5));
            });
        }
    }

    #[task(
        binds = USART1,
        shared = [
            console_rxbuf,
        ],
        local = [
            usart1_rx,
            usart1_tx,
            usart1_txbuf: ConstGenericRingBuffer<u8, LOG_BUFFER_SIZE> =
                    ConstGenericRingBuffer::new(),
        ])
    ]
    fn usart1(mut cx: usart1::Context) {
        // Check if there is something to receive, and if so, receive it into
        // somewhere
        if let Ok(b) = cx.local.usart1_rx.read() {
            trace!("USART1/console: Received: {:?}", b);
            //cx.local.usart1_txbuf.push(b); // Echo
            cx.shared.console_rxbuf.lock(|rxbuf| {
                rxbuf.push(b);
            });
        }
        if cx.local.usart1_txbuf.is_empty() {
            // Copy MULTI_LOGGER's buffer to usart1_txbuf
            // NOTE: This assumes there are only single-byte characters in the
            // buffer. Otherwise it won't fully fit in our byte-based usart1_txbuf
            let logger_usart1_txbuf_option = MULTI_LOGGER.get_uart_buffer();
            if let Some(logger_usart1_txbuf) = logger_usart1_txbuf_option {
                for b in logger_usart1_txbuf.bytes() {
                    cx.local.usart1_txbuf.push(b);
                }
            }
            if cx.local.usart1_txbuf.is_empty() {
                cx.local.usart1_tx.unlisten();
            }
        }
        if let Some(b) = cx.local.usart1_txbuf.front() {
            match cx.local.usart1_tx.write(*b) {
                Ok(_) => {
                    cx.local.usart1_txbuf.dequeue();
                }
                Err(_) => {}
            }
        }
        if !cx.local.usart1_txbuf.is_empty() {
            cx.local.usart1_tx.listen();
        }
    }

    /*#[task(
        binds = USART3,
        shared = [mainboard_rxbuf, mainboard_txbuf],
        local = [
            usart3_rx,
            usart3_tx,
        ])
    ]
    fn usart3(mut cx: usart3::Context) {
        // Receive to buffer
        if let Ok(b) = cx.local.usart3_rx.read() {
            trace!("USART3/mainboard: Received: {:?}", b);
            cx.shared.mainboard_rxbuf.lock(|rxbuf| {
                rxbuf.push(b);
            });
        }
        // Transmit from buffer
        cx.shared.mainboard_txbuf.lock(|txbuf| {
            if let Some(b) = txbuf.front() {
                match cx.local.usart3_tx.write(*b) {
                    Ok(_) => {
                        txbuf.dequeue();
                    },
                    Err(_) => {},
                }
            }
            if txbuf.is_empty() {
                cx.local.usart3_tx.unlisten();
            } else {
                cx.local.usart3_tx.listen();
            }
        });
    }*/

    #[task(
        binds = USART2,
        shared = [sim7600_rxbuf, sim7600_txbuf],
        local = [
            usart2_rx,
            usart2_tx,
        ])
    ]
    fn usart2(mut cx: usart2::Context) {
        // Receive to buffer
        if let Ok(b) = cx.local.usart2_rx.read() {
            //trace!("USART2/SIM7600: Received: {:?}", b);
            cx.shared.sim7600_rxbuf.lock(|rxbuf| {
                rxbuf.push(b);
            });
        }
        // Transmit from buffer
        cx.shared.sim7600_txbuf.lock(|txbuf| {
            if let Some(b) = txbuf.front() {
                match cx.local.usart2_tx.write(*b) {
                    Ok(_) => {
                        txbuf.dequeue();
                    }
                    Err(_) => {}
                }
            }
            if txbuf.is_empty() {
                cx.local.usart2_tx.unlisten();
            } else {
                cx.local.usart2_tx.listen();
            }
        });
    }

    #[task(
        binds = OTG_FS,
        shared = [
            usb_dev,
            usb_serial,
            console_rxbuf
        ],
        local = [
            usb_serial_txbuf: ConstGenericRingBuffer<u8, LOG_BUFFER_SIZE> =
                    ConstGenericRingBuffer::new(),
        ],
    )]
    fn otg_fs_int(cx: otg_fs_int::Context) {
        let otg_fs_int::SharedResources {
            __rtic_internal_marker,
            mut usb_dev,
            mut usb_serial,
            mut console_rxbuf,
        } = cx.shared;

        // Fill up usb_serial_txbuf
        if cx.local.usb_serial_txbuf.is_empty() {
            // NOTE: This assumes there are only single-byte characters in the
            // buffer. Otherwise it won't fully fit in our byte-based usb_serial_txbuf
            let logger_usb_serial_txbuf_option = MULTI_LOGGER.get_usb_buffer();
            if let Some(logger_usb_serial_txbuf) = logger_usb_serial_txbuf_option {
                for b in logger_usb_serial_txbuf.bytes() {
                    cx.local.usb_serial_txbuf.push(b);
                }
            }
        }

        // Write
        (&mut usb_serial).lock(|usb_serial| {
            if let Some(b) = cx.local.usb_serial_txbuf.front() {
                let buf: [u8; 1] = [*b];
                match usb_serial.write(&buf) {
                    Ok(n_written) => {
                        for _ in 0..n_written {
                            cx.local.usb_serial_txbuf.dequeue();
                        }
                    }
                    _ => {}
                }
            }
        });

        // Read
        (&mut usb_dev, &mut usb_serial, &mut console_rxbuf).lock(
            |usb_dev, usb_serial, console_rxbuf| {
                if usb_dev.poll(&mut [usb_serial]) {
                    let mut buf = [0u8; 64];
                    match usb_serial.read(&mut buf) {
                        Ok(count) if count > 0 => {
                            for i in 0..count {
                                //cx.local.usb_serial_txbuf.push(buf[i]); // Echo
                                console_rxbuf.push(buf[i]);
                            }
                        }
                        _ => {}
                    }
                }
            },
        );
    }

    #[task(
        binds = CAN1_RX0,
        shared = [
            can1,
            can_rx_buf,
        ]
    )]
    fn can1_rx0(cx: can1_rx0::Context) {
        (cx.shared.can1, cx.shared.can_rx_buf).lock(|can1, can_rx_buf| {
            if let Ok(frame) = can1.receive() {
                trace!("CAN1 << {:?} {:?}", frame.id(), frame.data());
                can_rx_buf.push(frame);
            }
        });
    }

    #[task(
        binds = CAN1_RX1,
        shared = [
            can1,
            can_rx_buf,
        ]
    )]
    fn can1_rx1(cx: can1_rx1::Context) {
        (cx.shared.can1, cx.shared.can_rx_buf).lock(|can1, can_rx_buf| {
            if let Ok(frame) = can1.receive() {
                trace!("CAN1 << {:?} {:?}", frame.id(), frame.data());
                can_rx_buf.push(frame);
            }
        });
    }

    #[task(
        binds = CAN1_TX,
        shared = [
            can1,
            can_tx_buf,
        ]
    )]
    fn can1_tx(cx: can1_tx::Context) {
        (cx.shared.can1, cx.shared.can_tx_buf).lock(|can1, can_tx_buf| {
            can1.clear_tx_interrupt();
            if let Some(frame) = can_tx_buf.dequeue() {
                trace!("-!- CAN1 >> {:?} {:?}", frame.id(), frame.data());
                let _ = can1.transmit(&frame);
            }
        });
    }
}

const PANIC_TEXT_STYLE: mono_font::MonoTextStyle<Rgb565> = mono_font::MonoTextStyleBuilder::new()
    .font(&mono_font::iso_8859_10::FONT_10X20)
    .text_color(Rgb565::WHITE)
    .background_color(Rgb565::CSS_DARK_RED)
    .build();

const PANIC_ACTION_STYLE: mono_font::MonoTextStyle<Rgb565> = mono_font::MonoTextStyleBuilder::new()
    .font(&mono_font::iso_8859_10::FONT_10X20)
    .text_color(Rgb565::CSS_FUCHSIA)
    .background_color(Rgb565::CSS_DARK_RED)
    .build();

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(panic_tx) = unsafe { PANIC_TX.as_mut() } {
        _ = panic_tx.write_str("\r\n");
        if let Some(location) = info.location() {
            _ = write!(
                panic_tx,
                "Panic at {}:{}:{}: ",
                location.file(),
                location.line(),
                location.column()
            );
        }
        _ = core::fmt::write(panic_tx, format_args!("{}", info.message()));
        _ = panic_tx.write_str("\r\n\r\n");
        _ = panic_tx.flush();

        // Wait for some time so that it all actually gets printed out
        long_busywait();
    }

    if let Some(display) = unsafe { PANIC_DISPLAY.as_mut() } {
        display.clear(Rgb565::CSS_DARK_RED).unwrap();
        {
            Text::with_alignment(
                "Panic at ",
                Point::new(0, 20),
                PANIC_TEXT_STYLE,
                eg::text::Alignment::Left,
            )
            .draw(display)
            .unwrap();
        }

        {
            Text::with_alignment(
                "Boot",
                Point::new(0, 239),
                PANIC_ACTION_STYLE,
                eg::text::Alignment::Left,
            )
            .draw(display)
            .unwrap();
        }

        {
            Text::with_alignment(
                "DFU mode",
                Point::new(160, 239),
                PANIC_ACTION_STYLE,
                eg::text::Alignment::Center,
            )
            .draw(display)
            .unwrap();
        }

        if let Some(location) = info.location() {
            let mut text: ArrayString<32> = ArrayString::new();
            text.push_str(&str_format!(
                fixedstr::str32,
                "{}:{}:{}:",
                location.file(),
                location.line(),
                location.column()
            ));
            Text::with_alignment(
                &text,
                Point::new(0, 40),
                PANIC_TEXT_STYLE,
                eg::text::Alignment::Left,
            )
            .draw(display)
            .unwrap();
        }

        {
            let mut text: ArrayString<200> = ArrayString::new();
            _ = core::fmt::write(&mut text, format_args!("{}", info.message()));
            let mut row = 0;
            let mut col = 0;
            for (i, c) in text.chars().enumerate() {
                let mut s: ArrayString<1> = ArrayString::new();
                s.push(c);
                Text::with_alignment(
                    &s,
                    Point::new(col * 10, 80 + row as i32 * 20),
                    PANIC_TEXT_STYLE,
                    eg::text::Alignment::Left,
                )
                .draw(display)
                .unwrap();
                if i % 32 == 31 {
                    row += 1;
                    col = 0;
                } else {
                    col += 1;
                }
            }
        }

        // Wait for some time so that someone has time to see it on the display
        for i in 0..100 {
            let mut text: ArrayString<32> = ArrayString::new();
            text.push_str(&str_format!(fixedstr::str32, "{} / {}", i, 100));
            Text::with_alignment(
                &text,
                Point::new(319, 239),
                PANIC_TEXT_STYLE,
                eg::text::Alignment::Right,
            )
            .draw(display)
            .unwrap();

            long_busywait();

            if let Some(button1_pin) = unsafe { PANIC_BUTTON1_PIN.as_mut() } {
                if button1_pin.is_low() {
                    cortex_m::peripheral::SCB::sys_reset();
                }
            }

            if let Some(button3_pin) = unsafe { PANIC_BUTTON3_PIN.as_mut() } {
                if button3_pin.is_low() {
                    if let Some(boot0_control_pin) = unsafe { PANIC_BOOT0_CONTROL_PIN.as_mut() } {
                        boot0_control_pin.set_high();
                        long_busywait();
                        cortex_m::peripheral::SCB::sys_reset();
                    }
                }
            }
        }
    }

    cortex_m::peripheral::SCB::sys_reset();
}

fn short_busywait() {
    for _ in 0..20000 {
        cortex_m::asm::nop();
    }
}
fn medium_busywait() {
    for _ in 0..1000000 {
        cortex_m::asm::nop();
    }
}
fn long_busywait() {
    for _ in 0..10000000 {
        cortex_m::asm::nop();
    }
}
