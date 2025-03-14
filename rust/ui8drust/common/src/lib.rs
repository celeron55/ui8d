#![no_std]

pub mod sim7600;
pub use sim7600::*;
pub mod command_accumulator;

pub mod log_display;
pub use log_display::LogDisplay;

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
}

#[derive(Debug, Clone, Copy)]
pub enum DigitalOutput {
    Wakeup,
    Pwmout1, // TODO: Convert to actual PWM output
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

pub struct Parameter<'a, ID> {
    pub id: ID,
    pub display_name: &'a str,
    pub value: f32,
    pub decimals: u8,
    pub unit: &'a str,
    pub can_map: Option<CanMap>,
    pub report_map: Option<ReportMap<'a>>,
    // TODO: Timeout
}

impl<'a, ID> Parameter<'a, ID> {
    pub const fn new(
        id: ID,
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
        }
    }
}

