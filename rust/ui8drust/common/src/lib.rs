#![no_std]

pub mod sim7600;
pub use sim7600::*;
pub mod command_accumulator;

pub mod log_display;
pub use log_display::LogDisplay;

pub mod http;

pub extern crate bxcan;
pub extern crate embedded_graphics;
pub extern crate log;
pub extern crate profont;

use arrayvec::ArrayString;
use bxcan::StandardId;
use embedded_graphics as eg;
use embedded_graphics::{mono_font, pixelcolor::*, prelude::*};
use fixedstr::str_format;
use int_enum::IntEnum;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use ringbuffer::RingBuffer;
use bitvec::prelude::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Button {
    Button1,
    Button2,
    Button3,
    Button4,
    Button5,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ButtonEvent {
    ButtonPress(Button),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct HttpResponse {
    pub status_code: u16,
    pub body: ArrayString<1000>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum HttpFailReason {
    Unknown,
    InternalTimeout,
    ServerTimeout,
    InternalError,
    ServerError,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum HttpUpdateStatus {
    NotStarted,
    Processing,
    Failed(HttpFailReason),
    Finished(HttpResponse),
}

#[derive(Debug, Clone, Copy)]
pub enum AnalogInput {
    AuxVoltage,
    PcbT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DigitalOutput {
    Wakeup,
    Pwmout1, // TODO: Convert to actual PWM output
    Sim7600PowerInhibit,
}

pub trait HardwareInterface {
    fn millis(&mut self) -> u64;

    fn display_clear(&mut self, color: Rgb565);

    fn display_draw_text(
        &mut self,
        text: &str,
        p: Point,
        style: mono_font::MonoTextStyle<Rgb565>,
        alignment: eg::text::Alignment,
    );

    fn reboot(&mut self);
    fn activate_dfu(&mut self);

    fn http_get_start(&mut self, url: &str);
    fn http_get_update(&mut self) -> HttpUpdateStatus;
    fn http_get_stop(&mut self);

    fn send_can(&mut self, frame: bxcan::Frame);

    fn get_analog_input(&mut self, input: AnalogInput) -> f32;

    fn set_digital_output(&mut self, output: DigitalOutput, value: bool);
}

// Parameter definitions

pub enum CanBitSelection {
    Bit(u8),
    BeUnsigned(u8, u8),
    LeUnsigned(u8, u8),
    BeSigned(u8, u8),
    LeSigned(u8, u8),
    Uint8(u8),
    Int8(u8),
    Function(fn(&[u8]) -> f32),
}

pub struct CanMap {
    pub id: bxcan::Id,
    pub bits: CanBitSelection,
    pub scale: f32,
}

pub struct ReportMap<'a> {
    pub name: &'a str,
    pub decimals: u8,
    pub scale: f32,
}

pub struct Parameter<'a> {
    pub id: usize,
    pub display_name: &'a str,
    pub value: f32,
    pub decimals: u8,
    pub unit: &'a str,
    pub can_map: Option<CanMap>,
    pub report_map: Option<ReportMap<'a>>,
    pub update_timestamp: u64,
}

impl<'a> Parameter<'a> {
    pub const fn new(
        id: usize,
        display_name: &'a str,
        value: f32,
        decimals: u8,
        unit: &'a str,
        can_map: Option<CanMap>,
        report_map: Option<ReportMap<'a>>,
    ) -> Self {
        Self {
            id: id,
            display_name: display_name,
            value: value,
            decimals: decimals,
            unit: unit,
            can_map: can_map,
            report_map: report_map,
            update_timestamp: 0,
        }
    }
    pub fn set_value(&mut self, value: f32, millis: u64) {
        self.value = value;
        self.update_timestamp = millis;
    }
}

pub static mut PARAMETERS: Option<&'static mut [Parameter<'static>]> = None;

pub fn set_parameters(params: &'static mut [Parameter<'static>]) {
    unsafe {
        PARAMETERS = Some(params);
    }
}

#[macro_export] macro_rules! define_parameters {
    ($($name:ident {
        display_name: $display_name:expr,
        $(value: $value:expr,)?
        $(decimals: $decimals:expr,)?
        unit: $unit:expr,
        $(can_map: $can_map:expr,)?
        $(report_map: $report_map:expr,)?
    }),* $(,)?) => {
        pub const NUM_PARAMETERS: usize = {
            let mut count = 0;
            $(let _ = stringify!($name); count += 1;)*
            count
        };

        #[repr(usize)]
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum ParameterId {
            $($name),*
        }

        // Implement conversion from usize to ParameterId using array lookup
        const PARAMETER_IDS: [ParameterId; NUM_PARAMETERS] = [
            $(ParameterId::$name),*
        ];
        impl ParameterId {
            pub fn from_usize(value: usize) -> Option<Self> {
                if value < PARAMETER_IDS.len() {
                    Some(PARAMETER_IDS[value])
                } else {
                    None
                }
            }
        }

        pub static mut PARAMETERS: [Parameter; NUM_PARAMETERS] = [
            $(
                Parameter {
                    id: ParameterId::$name as usize,
                    display_name: $display_name,
                    value: {
                        #[allow(unused_variables)]
                        let value: f32 = f32::NAN;
                        $(let value = $value;)?
                        value
                    },
                    decimals: {
                        #[allow(unused_variables)]
                        let decimals: u8 = 0;
                        $(let decimals = $decimals;)?
                        decimals
                    },
                    unit: $unit,
                    can_map: {
                        #[allow(unused_variables)]
                        let can_map: Option<CanMap> = None;
                        $(let can_map = Some($can_map);)?
                        can_map
                    },
                    report_map: {
                        #[allow(unused_variables)]
                        let report_map: Option<ReportMap> = None;
                        $(let report_map = Some($report_map);)?
                        report_map
                    },
                    update_timestamp: 0,
                }
            ),*
        ];

        // Accessor using ParameterId enum
		pub fn get_parameter(id: ParameterId) -> &'static mut Parameter<'static> {
		    get_parameter_id(id as usize)
		}

        // Initialization function: Call this at start of main() or whatever
        pub fn init_parameters() {
            unsafe {
                $crate::set_parameters(&mut PARAMETERS);
            }
        }
    };
}

pub fn get_parameters() -> &'static mut [Parameter<'static>] {
    unsafe {
        PARAMETERS.as_mut().expect("Parameters not initialized")
    }
}

pub fn get_parameter_id(id: usize) -> &'static mut Parameter<'static> {
    unsafe {
        return &mut PARAMETERS.as_mut().expect("Parameters not initialized")[id];
    }
}

pub fn update_parameters_on_can(frame: bxcan::Frame, millis: u64) {
    for (i, param) in get_parameters().iter_mut().enumerate() {
        if let Some(can_map) = &param.can_map {
            if let Some(data) = frame.data() {
                if can_map.id == frame.id() {
                    match can_map.bits {
                        CanBitSelection::Bit(bit_i) => {
                            let byte = data[(bit_i as usize) / 8];
                            let bit_in_byte = bit_i % 8;
                            let mask = 1 << bit_in_byte;
                            param.set_value(
                                    ((byte & mask) >> bit_in_byte) as f32 * can_map.scale,
                                    millis);
                        }
                        CanBitSelection::BeUnsigned(i0, len) => {
                            let bits = data.view_bits::<Msb0>();
                            let raw = bits[i0 as usize .. (i0+len) as usize].load_be::<u64>();
                            param.set_value(raw as f32 * can_map.scale, millis);
                        }
                        CanBitSelection::LeUnsigned(i0, len) => {
                            let bits = data.view_bits::<Lsb0>();
                            let raw = bits[i0 as usize .. (i0+len) as usize].load_le::<u64>();
                            param.set_value(raw as f32 * can_map.scale, millis);
                        }
                        CanBitSelection::BeSigned(i0, len) => {
                            let bits = data.view_bits::<Msb0>();
                            let raw = bits[i0 as usize .. (i0+len) as usize].load_be::<i64>();
                            param.set_value(raw as f32 * can_map.scale, millis);
                        }
                        CanBitSelection::LeSigned(i0, len) => {
                            let bits = data.view_bits::<Lsb0>();
                            let raw = bits[i0 as usize .. (i0+len) as usize].load_le::<i64>();
                            param.set_value(raw as f32 * can_map.scale, millis);
                        }
                        CanBitSelection::Uint8(byte_i) => {
                            param.set_value((data[byte_i as usize] as u8) as
                                    f32 * can_map.scale,
                                millis);
                        }
                        CanBitSelection::Int8(byte_i) => {
                            param.set_value((data[byte_i as usize] as i8) as
                                    f32 * can_map.scale,
                                millis);
                        }
                        CanBitSelection::Function(function) => {
                            param.set_value(function(data) * can_map.scale,
                                millis);
                        }
                    }
                }
            }
        }
    }
}
