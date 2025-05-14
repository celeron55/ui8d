#![no_std]

// You need to supply this at build time
// Example: "http://example.com/report?id=test&"
const base_url: &str = env!("BASE_URL");

const CHARGE_COMPLETE_VOLTAGE_SETTING_MV: u16 = 4160; // Should be divisible by 20

use common::*;

pub mod can_simulator;
pub mod parameters;
use parameters::*;

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
use core::fmt::Write;

// View definitions

#[derive(PartialEq)]
struct View {
    pub on_update: fn(redraw: bool, state: &mut MainState, hw: &mut dyn HardwareInterface),
    pub on_button:
        fn(event: ButtonEvent, state: &mut MainState, hw: &mut dyn HardwareInterface) -> bool,
}

const TEXT_TOP_ROW_Y: i32 = 19;
const TEXT_ACTION_ROW_Y: i32 = 237;
const PARAM_ROW_HEIGHT: i32 = 26;

const TEXT_STYLE_BRAND: mono_font::MonoTextStyle<Rgb565> =
    mono_font::MonoTextStyle::new(&profont::PROFONT_9_POINT, Rgb565::CSS_GRAY);

const TEXT_STYLE_TITLE: mono_font::MonoTextStyle<Rgb565> = mono_font::MonoTextStyleBuilder::new()
    .font(&profont::PROFONT_18_POINT)
    .text_color(Rgb565::CSS_FUCHSIA)
    .background_color(Rgb565::BLACK)
    .build();

const TEXT_STYLE_UI: mono_font::MonoTextStyle<Rgb565> = mono_font::MonoTextStyleBuilder::new()
    .font(&profont::PROFONT_24_POINT)
    .text_color(Rgb565::WHITE)
    .background_color(Rgb565::BLACK)
    .build();

const TEXT_STYLE_ERROR: mono_font::MonoTextStyle<Rgb565> = mono_font::MonoTextStyleBuilder::new()
    .font(&profont::PROFONT_18_POINT)
    .text_color(Rgb565::RED)
    .background_color(Rgb565::BLACK)
    .build();

const TEXT_STYLE_WARNING_NOTIFICATION: mono_font::MonoTextStyle<Rgb565> =
    mono_font::MonoTextStyleBuilder::new()
        .font(&profont::PROFONT_18_POINT)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::RED)
        .build();

const TEXT_STYLE_PARAMETER_NAME: mono_font::MonoTextStyle<Rgb565> =
    mono_font::MonoTextStyleBuilder::new()
        .font(&profont::PROFONT_18_POINT)
        .text_color(Rgb565::CSS_LIGHT_CYAN)
        .background_color(Rgb565::BLACK)
        .build();

const TEXT_STYLE_PARAMETER_UNIT: mono_font::MonoTextStyle<Rgb565> =
    mono_font::MonoTextStyleBuilder::new()
        .font(&profont::PROFONT_18_POINT)
        .text_color(Rgb565::CSS_LIGHT_YELLOW)
        .background_color(Rgb565::BLACK)
        .build();

const TEXT_STYLE_PARAMETER_VALUE: mono_font::MonoTextStyle<Rgb565> =
    mono_font::MonoTextStyleBuilder::new()
        .font(&profont::PROFONT_24_POINT)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::BLACK)
        .build();

const TEXT_STYLE_BUTTON_ACTION: mono_font::MonoTextStyle<Rgb565> =
    mono_font::MonoTextStyleBuilder::new()
        .font(&profont::PROFONT_18_POINT)
        .text_color(Rgb565::CSS_FUCHSIA)
        .background_color(Rgb565::BLACK)
        .build();

const TEXT_STYLE_BUTTON_ACTION_ACTIVE: mono_font::MonoTextStyle<Rgb565> =
    mono_font::MonoTextStyleBuilder::new()
        .font(&profont::PROFONT_18_POINT)
        .text_color(Rgb565::CSS_LIME)
        .background_color(Rgb565::BLACK)
        .underline()
        .build();

const TEXT_STYLE_LOG: mono_font::MonoTextStyle<Rgb565> = mono_font::MonoTextStyleBuilder::new()
    .font(&profont::PROFONT_9_POINT)
    .text_color(Rgb565::WHITE)
    .background_color(Rgb565::BLACK)
    .build();

pub fn draw_brand_background(hw: &mut dyn HardwareInterface) {
    hw.display_clear(Rgb565::BLACK);
}

pub fn draw_button_action(i: usize, text: &str, active: bool, hw: &mut dyn HardwareInterface) {
    hw.display_draw_text(
        text,
        Point::new(i as i32 * 75, TEXT_ACTION_ROW_Y),
        if active {
            TEXT_STYLE_BUTTON_ACTION_ACTIVE
        } else {
            TEXT_STYLE_BUTTON_ACTION
        },
        eg::text::Alignment::Left,
    );
}

pub fn draw_view_number(view_i: usize, hw: &mut dyn HardwareInterface) {
    hw.display_draw_text(
        &str_format!(fixedstr::str8, "{}", view_i + 1),
        Point::new(3 as i32 * 75 + 75 / 2, TEXT_ACTION_ROW_Y),
        TEXT_STYLE_BUTTON_ACTION,
        eg::text::Alignment::Left,
    );
}

pub fn draw_parameter_text(
    display_name: &str,
    text: &str,
    unit: &str,
    y: i32,
    redraw: bool,
    hw: &mut dyn HardwareInterface,
) {
    if redraw {
        hw.display_draw_text(
            display_name,
            Point::new(0, y),
            TEXT_STYLE_PARAMETER_NAME,
            eg::text::Alignment::Left,
        );
        hw.display_draw_text(
            unit,
            Point::new(260, y),
            TEXT_STYLE_PARAMETER_UNIT,
            eg::text::Alignment::Left,
        );
    }
    hw.display_draw_text(
        &text,
        Point::new(255, y),
        TEXT_STYLE_PARAMETER_VALUE,
        eg::text::Alignment::Right,
    );
}

pub fn draw_parameter_dual_text(
    display_name: &str,
    text1: &str,
    unit1: &str,
    text2: &str,
    unit2: &str,
    y: i32,
    redraw: bool,
    hw: &mut dyn HardwareInterface,
) {
    if redraw {
        hw.display_draw_text(
            display_name,
            Point::new(0, y),
            TEXT_STYLE_PARAMETER_NAME,
            eg::text::Alignment::Left,
        );
        hw.display_draw_text(
            unit1,
            Point::new(160, y),
            TEXT_STYLE_PARAMETER_UNIT,
            eg::text::Alignment::Left,
        );
        hw.display_draw_text(
            unit2,
            Point::new(260, y),
            TEXT_STYLE_PARAMETER_UNIT,
            eg::text::Alignment::Left,
        );
    }
    hw.display_draw_text(
        &text1,
        Point::new(155, y),
        TEXT_STYLE_PARAMETER_VALUE,
        eg::text::Alignment::Right,
    );
    hw.display_draw_text(
        &text2,
        Point::new(255, y),
        TEXT_STYLE_PARAMETER_VALUE,
        eg::text::Alignment::Right,
    );
}

pub fn draw_parameter_raw(
    display_name: &str,
    value: f32,
    decimals: usize,
    unit: &str,
    y: i32,
    redraw: bool,
    hw: &mut dyn HardwareInterface,
) {
    let mut text: ArrayString<10> = ArrayString::new();
    if value.is_nan() {
        text.push_str("     -");
    } else {
        text.push_str(&str_format!(fixedstr::str16, "{: >6.*}", decimals, value));
    }

    draw_parameter_text(display_name, &text, unit, y, redraw, hw);
}

pub fn draw_parameter_dual_raw(
    display_name: &str,
    value1: f32,
    decimals1: usize,
    unit1: &str,
    value2: f32,
    decimals2: usize,
    unit2: &str,
    y: i32,
    redraw: bool,
    hw: &mut dyn HardwareInterface,
) {
    let mut text1: ArrayString<10> = ArrayString::new();
    if value1.is_nan() {
        text1.push_str("   -");
    } else {
        text1.push_str(&str_format!(fixedstr::str16, "{: >4.*}", decimals1, value1));
    }

    let mut text2: ArrayString<10> = ArrayString::new();
    if value2.is_nan() {
        text2.push_str("   -");
    } else {
        text2.push_str(&str_format!(fixedstr::str16, "{: >4.*}", decimals2, value2));
    }

    draw_parameter_dual_text(display_name, &text1, unit1, &text2, unit2, y, redraw, hw);
}

pub fn draw_parameter(id: ParameterId, y: i32, redraw: bool, hw: &mut dyn HardwareInterface) {
    let param = get_parameter(id);

    draw_parameter_raw(
        param.display_name,
        param.value,
        param.decimals as usize,
        param.unit,
        y,
        redraw,
        hw,
    );
}

pub fn draw_parameter_dual_custom_midstring(
    display_name: &str,
    id1: ParameterId,
    midstring: &str,
    id2: ParameterId,
    y: i32,
    redraw: bool,
    hw: &mut dyn HardwareInterface,
) {
    let param1 = get_parameter(id1);
    let param2 = get_parameter(id2);

    draw_parameter_dual_raw(
        display_name,
        param1.value,
        param1.decimals as usize,
        midstring,
        param2.value,
        param2.decimals as usize,
        param2.unit,
        y,
        redraw,
        hw,
    );
}

pub fn draw_parameter_dual(
    display_name: &str,
    id1: ParameterId,
    id2: ParameterId,
    y: i32,
    redraw: bool,
    hw: &mut dyn HardwareInterface,
) {
    let param1 = get_parameter(id1);
    let param2 = get_parameter(id2);

    draw_parameter_dual_raw(
        display_name,
        param1.value,
        param1.decimals as usize,
        param1.unit,
        param2.value,
        param2.decimals as usize,
        param2.unit,
        y,
        redraw,
        hw,
    );
}

#[derive(Debug, PartialEq)]
enum Warning {
    None,
    Test,
    AuxVoltageLow,
    BatteryHot,
    BatteryCriticallyLow,
    BatteryCriticallyHigh,
    HeaterOverTemperature,
    PrechargeFailed,
    ObcCpMismatch,
    ObcDcvMismatch,
    DcdcDown,
    DcdcAuxMismatch,
    DcdcZeroCurrent,
    InverterHot,
    MotorHot,
    IpdmMcReqFail,
    IpdmGroup1OC,
    IpdmGroup2OC,
    IpdmGroup3OC,
    IpdmGroup4OC,
}

impl Warning {
    fn to_text<T: Write>(&self, s: &mut T) {
        match self {
            Warning::None => write!(s, ""),
            Warning::Test => write!(s, "Test warning"),
            Warning::AuxVoltageLow => write!(s, "Aux battery low"),
            Warning::InverterHot => write!(s, "Inverter hot"),
            Warning::MotorHot => write!(s, "Motor hot"),
            a => core::fmt::write(s, format_args!("{:?}", self)),
        };
    }
}

fn generate_warning(hw: &mut dyn HardwareInterface) -> Warning {
    if get_parameter(ParameterId::InverterT).value >= 60.0 {
        Warning::InverterHot
    } else if get_parameter(ParameterId::MotorT).value >= 60.0 {
        Warning::MotorHot
    } else if get_parameter(ParameterId::BatteryTMax).value >= 50.0 {
        Warning::BatteryHot
    } else if get_parameter(ParameterId::BatteryVMin).value <= 3.0 {
        Warning::BatteryCriticallyLow
    } else if get_parameter(ParameterId::BatteryVMax).value >= 4.20 {
        Warning::BatteryCriticallyHigh
    } else if get_parameter(ParameterId::AuxVoltage).value <= 11.5 {
        Warning::AuxVoltageLow
    } else if get_parameter(ParameterId::HeaterT).value >= 100.0 {
        Warning::HeaterOverTemperature
    } else if get_parameter(ParameterId::PrechargeFailed).value >= 0.5 {
        Warning::PrechargeFailed
    } else if get_parameter(ParameterId::MainContactor).value >= 0.5 &&
            (get_parameter(ParameterId::ObcDcv).value < 150.0 ||
                get_parameter(ParameterId::ObcDcv).value > 400.0) {
        Warning::ObcDcvMismatch
    } else if (get_parameter(ParameterId::FoccciCPPWM).value -
            get_parameter(ParameterId::ObcEvsePwm).value).abs() > 2.0 {
        Warning::ObcCpMismatch
    } else if get_parameter(ParameterId::MainContactor).value >= 0.5 &&
            get_parameter(ParameterId::DcdcStatus).value != 0x22 as f32 {
        Warning::DcdcDown
    } else if (get_parameter(ParameterId::DcdcAuxVoltage).value -
            get_parameter(ParameterId::AuxVoltage).value).abs() > 1.0 {
        Warning::DcdcAuxMismatch
    } else if get_parameter(ParameterId::IpdmReqMC).value > 0.5 &&
            get_parameter(ParameterId::MainContactor).value < 0.5 &&
            get_parameter(ParameterId::Precharging).value < 0.5 {
        Warning::IpdmMcReqFail
    } else if get_parameter(ParameterId::IpdmGroup1OC).value > 0.5 {
        Warning::IpdmGroup1OC
    } else if get_parameter(ParameterId::IpdmGroup2OC).value > 0.5 {
        Warning::IpdmGroup2OC
    } else if get_parameter(ParameterId::IpdmGroup3OC).value > 0.5 {
        Warning::IpdmGroup3OC
    } else if get_parameter(ParameterId::IpdmGroup4OC).value > 0.5 {
        Warning::IpdmGroup4OC
    } else if get_parameter(ParameterId::MainContactor).value >= 0.5 &&
            get_parameter(ParameterId::DcdcCurrent).value < 0.2 as f32 {
        Warning::DcdcZeroCurrent
    } else if hw.millis() < 2500 {
        Warning::Test
    } else {
        Warning::None
    }
}

static mut main_view_drawn_cruise_requested: f32 = f32::NAN;
static mut main_view_drawn_cruise_active: f32 = f32::NAN;
static mut main_view_drawn_warning: Warning = Warning::None;

static main_view: View = View {
    on_update: |redraw0: bool, state: &mut MainState, hw: &mut dyn HardwareInterface| {
        let mut redraw = redraw0;

        let cruise_changed = unsafe {
            main_view_drawn_cruise_requested !=
                get_parameter(ParameterId::CruiseRequested).value ||
            main_view_drawn_cruise_active !=
                get_parameter(ParameterId::CruiseActive).value };

        let warning = generate_warning(hw);

        let warning_changed = unsafe { main_view_drawn_warning != warning };

        if warning_changed {
            redraw = true;
        }

        if redraw {
            draw_brand_background(hw);
            draw_view_number(state.current_view, hw);
            draw_button_action(0, "Reboot", false, hw);
        }

        if redraw || cruise_changed {
            unsafe {
                main_view_drawn_cruise_requested =
                        get_parameter(ParameterId::CruiseRequested).value;
                main_view_drawn_cruise_active =
                        get_parameter(ParameterId::CruiseActive).value;
            };
            draw_button_action(1,
                if get_parameter(ParameterId::CruiseRequested).value > 0.5 {
                    if get_parameter(ParameterId::CruiseActive).value ==
                            get_parameter(ParameterId::CruiseRequested).value {
                        "Cruis"
                    } else {
                        "Crui?"
                    }
                } else {
                    "Cruis"
                },
                get_parameter(ParameterId::CruiseActive).value > 0.5 ||
                        get_parameter(ParameterId::CruiseRequested).value > 0.5,
                hw);
        }

        if redraw {
            //draw_button_action(1, "BHeat", true, hw);
            {
                let mut text: ArrayString<10> = ArrayString::new();
                text.push_str(&str_format!(fixedstr::str16, "{: >2.*}A", 0, get_parameter(ParameterId::AcChargeCurrentSetting).value));
                draw_button_action(2, &text, get_parameter(ParameterId::AcChargeCurrentSetting).value >= 13.0, hw);
            }
            draw_button_action(3, "<", false, hw);
            draw_button_action(4, ">", false, hw);
        }

        draw_parameter_dual(
            "Range",
            ParameterId::Soc,
            ParameterId::RangeKm,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 0,
            redraw,
            hw,
        );
        draw_parameter_dual_custom_midstring(
            "Battery",
            ParameterId::BatteryTMin,
            " ..",
            ParameterId::BatteryTMax,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 1,
            redraw,
            hw,
        );
        draw_parameter_dual_custom_midstring(
            "",
            ParameterId::BatteryVMin,
            "..",
            ParameterId::BatteryVMax,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 2,
            redraw,
            hw,
        );
        /*draw_parameter(
            ParameterId::AllowedChargePower,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 2,
            redraw,
            hw,
        );*/
        /*draw_parameter_text(
            "Heat status",
            "BDE",
            "",
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 3,
            redraw,
            hw,
        );*/
        draw_parameter_dual(
            "Heater",
            ParameterId::HeaterT,
            ParameterId::HeaterPowerPercent,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 3,
            redraw,
            hw,
        );
        /*draw_parameter_dual(
            "Trip",
            ParameterId::TripKm,
            ParameterId::TripConsumption,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 4,
            redraw,
            hw,
        );*/
        draw_parameter(
            ParameterId::CabinT,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 4,
            redraw,
            hw,
        );
        /*draw_parameter_dual(
            "Recent",
            ParameterId::RecentKm,
            ParameterId::RecentConsumption,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 5,
            redraw,
            hw,
        );*/
        draw_parameter(
            ParameterId::ChargePower,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 5,
            redraw,
            hw,
        );
        /*draw_parameter(
            ParameterId::TicksMs,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 6,
            redraw,
            hw,
        );*/
        draw_parameter_dual(
            "OBC",
            ParameterId::ObcDcv,
            ParameterId::ObcDcc,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 6,
            redraw,
            hw,
        );

        if warning == Warning::None {
            draw_parameter(
                ParameterId::AuxVoltage,
                TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 7,
                redraw,
                hw,
            );
        } else {
            let mut warning_text: ArrayString<32> = ArrayString::new();
            warning.to_text(&mut warning_text);
            hw.display_draw_text(
                &warning_text,
                Point::new(320/2, TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 7),
                TEXT_STYLE_WARNING_NOTIFICATION,
                eg::text::Alignment::Center,
            );
        }
        unsafe {
            main_view_drawn_warning = warning;
        }
    },

    on_button: |event: ButtonEvent,
                state: &mut MainState,
                hw: &mut dyn HardwareInterface|
     -> bool {
        match event {
            ButtonEvent::ButtonPress(Button::Button1) => {
                hw.reboot();
                return true;
            }
            ButtonEvent::ButtonPress(Button::Button2) => {
                if get_parameter(ParameterId::CruiseRequested).value < 0.5 {
                    get_parameter(ParameterId::CruiseRequested).value = 1.0;
                    hw.send_can(bxcan::Frame::new_data(
                        bxcan::StandardId::new(0x320).unwrap(),
                        bxcan::Data::new(b"\x02\x00\x00\x00\x01\x00\x00\x00").unwrap()
                    ));
                } else {
                    get_parameter(ParameterId::CruiseRequested).value = 0.0;
                    hw.send_can(bxcan::Frame::new_data(
                        bxcan::StandardId::new(0x320).unwrap(),
                        bxcan::Data::new(b"\x02\x00\x00\x00\x00\x00\x00\x00").unwrap()
                    ));
                }
                return true;
            }
            ButtonEvent::ButtonPress(Button::Button3) => {
                if get_parameter(ParameterId::AcChargeCurrentSetting).value < 13.0 {
                    get_parameter(ParameterId::AcChargeCurrentSetting).value = 16.0;
                } else {
                    get_parameter(ParameterId::AcChargeCurrentSetting).value = 10.0;
                }
                return true;
            }
            ButtonEvent::ButtonPress(_) => {}
        }
        false
     },
};

const PARAMS_PER_PAGE: usize = 8;

fn draw_all_params_view_bg(state: &mut MainState, hw: &mut dyn HardwareInterface) {
    draw_brand_background(hw);
    hw.display_draw_text(
        &str_format!(fixedstr::str8, "{}", state.all_params_view_page + 1),
        Point::new((1.5 * 75.0) as i32, TEXT_ACTION_ROW_Y),
        TEXT_STYLE_BUTTON_ACTION_ACTIVE,
        eg::text::Alignment::Left,
    );
    draw_view_number(state.current_view, hw);
    draw_button_action(1, "<", true, hw);
    draw_button_action(2, ">", true, hw);
    draw_button_action(3, "<", false, hw);
    draw_button_action(4, ">", false, hw);
}

fn draw_all_params_view_fg(redraw: bool, state: &mut MainState, hw: &mut dyn HardwareInterface) {
    for i in 0..PARAMS_PER_PAGE {
        let i1 = state.all_params_view_page * PARAMS_PER_PAGE + i;
        if let Some(id) = ParameterId::from_usize(i1) {
            draw_parameter(
                ParameterId::from_usize(i1).unwrap(),
                TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * i as i32,
                redraw,
                hw,
            );
        }
    }
}

static all_params_view: View = View {
    on_update: |redraw: bool, state: &mut MainState, hw: &mut dyn HardwareInterface| {
        if redraw {
            draw_all_params_view_bg(state, hw);
        }

        draw_all_params_view_fg(redraw, state, hw);
    },

    on_button: |event: ButtonEvent,
                state: &mut MainState,
                hw: &mut dyn HardwareInterface|
     -> bool {
        match event {
            ButtonEvent::ButtonPress(Button::Button2) => {
                if state.all_params_view_page > 0 {
                    state.all_params_view_page -= 1;
                    return true;
                }
                return false;
            }
            ButtonEvent::ButtonPress(Button::Button3) => {
                if state.all_params_view_page < get_parameters().len() / PARAMS_PER_PAGE {
                    state.all_params_view_page += 1;
                    return true;
                }
                return false;
            }
            ButtonEvent::ButtonPress(_) => {}
        }
        false
    },
};

static log_view: View = View {
    on_update: |redraw: bool, state: &mut MainState, hw: &mut dyn HardwareInterface| {
        if redraw {
            draw_brand_background(hw);
            draw_view_number(state.current_view, hw);
            draw_button_action(3, "<", false, hw);
            draw_button_action(4, ">", false, hw);
        }

        let mut text: ArrayString<30> = ArrayString::new();
        text.push_str(&str_format!(fixedstr::str8, "{:>7}:", state.update_counter));

        hw.display_draw_text(
            &text,
            Point::new(200, TEXT_ACTION_ROW_Y),
            TEXT_STYLE_TITLE,
            eg::text::Alignment::Right,
        );

        for i in 0..log_display::NUM_LINES {
            if state.log_display.lines.len() <= i {
                break;
            }

            // Pad text with space in order to paint over the old line
            let mut text: ArrayString<{ log_display::LINE_MAX_LENGTH }> =
                ArrayString::from(&state.log_display.lines[i]).unwrap();
            while text.len() < log_display::LINE_MAX_LENGTH {
                _ = text.try_push(' ');
            }

            hw.display_draw_text(
                &text,
                Point::new(0, TEXT_TOP_ROW_Y - 8 + 10 * (i as i32)),
                TEXT_STYLE_LOG,
                eg::text::Alignment::Left,
            );
        }
    },

    on_button: |event: ButtonEvent,
                state: &mut MainState,
                hw: &mut dyn HardwareInterface|
     -> bool { false },
};

static mainboard_log_view: View = View {
    on_update: |redraw: bool, state: &mut MainState, hw: &mut dyn HardwareInterface| {
        if redraw {
            draw_brand_background(hw);
            draw_view_number(state.current_view, hw);
            draw_button_action(3, "<", false, hw);
            draw_button_action(4, ">", false, hw);
        }

        let mut text: ArrayString<30> = ArrayString::new();
        text.push_str(&str_format!(fixedstr::str8, "{:>7}:", state.update_counter));

        hw.display_draw_text(
            &text,
            Point::new(200, TEXT_ACTION_ROW_Y),
            TEXT_STYLE_TITLE,
            eg::text::Alignment::Right,
        );

        for i in 0..log_display::NUM_LINES {
            if state.mainboard_log_display.lines.len() <= i {
                break;
            }

            // Pad text with space in order to paint over the old line
            let mut text: ArrayString<{ log_display::LINE_MAX_LENGTH }> =
                ArrayString::from(&state.mainboard_log_display.lines[i]).unwrap();
            while text.len() < log_display::LINE_MAX_LENGTH {
                _ = text.try_push(' ');
            }

            hw.display_draw_text(
                &text,
                Point::new(0, TEXT_TOP_ROW_Y - 8 + 10 * (i as i32)),
                TEXT_STYLE_LOG,
                eg::text::Alignment::Left,
            );
        }
    },

    on_button: |event: ButtonEvent,
                state: &mut MainState,
                hw: &mut dyn HardwareInterface|
     -> bool { false },
};

static views: [&View; 4] = [&main_view, &all_params_view, &log_view, &mainboard_log_view];

pub struct MainState {
    update_counter: u32,
    log_display: LogDisplay,
    mainboard_log_display: LogDisplay,
    current_view: usize,
    log_can: bool,
    all_params_view_page: usize,
    last_millis: u64,
    dt_ms: u64,
    http_process: http::HttpProcess,
    last_hvac_power_can_send_millis: u64,
    last_hvac_power_output_wanted_off_millis: u64,
    last_charge_config_millis: u64,
}

impl MainState {
    pub fn new() -> Self {
        init_parameters();

        Self {
            update_counter: 0,
            log_display: LogDisplay::new(),
            mainboard_log_display: LogDisplay::new(),
            current_view: 0,
            log_can: false,
            all_params_view_page: 0,
            last_millis: 0,
            dt_ms: 0,
            http_process: http::HttpProcess::new(),
            last_hvac_power_can_send_millis: 0,
            last_hvac_power_output_wanted_off_millis: 0,
            last_charge_config_millis: 0,
        }
    }

    // This should be called at 20ms interval
    pub fn update(&mut self, hw: &mut dyn HardwareInterface) {
        // Timekeeping
        let millis = hw.millis();
        self.dt_ms = if millis > self.last_millis {
            millis - self.last_millis
        } else {
            0
        };

        self.update_parameters(hw);

        self.update_view(hw);

        self.update_hvac_power(hw);

        self.update_charge_config(hw);

        self.update_http(hw);

        self.last_millis = millis;
        self.update_counter += 1;
    }

    fn timeout_parameters(&mut self, hw: &mut dyn HardwareInterface) {
        for (i, param) in get_parameters().iter_mut().enumerate() {
            if param.can_map.is_some() && !param.value.is_nan() {
                let age_ms = hw.millis() - param.update_timestamp;
                if age_ms >= 5000 {
                    param.value = f32::NAN;
                }
            }
        }
    }

    fn update_parameters(&mut self, hw: &mut dyn HardwareInterface) {
        get_parameter(ParameterId::TicksMs).set_value(hw.millis() as f32, hw.millis());
        get_parameter(ParameterId::AuxVoltage).set_value(hw.get_analog_input(AnalogInput::AuxVoltage), hw.millis());
        get_parameter(ParameterId::CabinT).set_value(hw.get_analog_input(AnalogInput::PcbT) - 12.0, hw.millis());

        get_parameter(ParameterId::ChargePower).set_value(
            if get_parameter(ParameterId::CcsCurrent).value > 1.0 {
                get_parameter(ParameterId::CcsCurrent).value *
                        get_parameter(ParameterId::ObcDcv).value * 0.001
            } else if get_parameter(ParameterId::ChademoCurrent).value > 1.0 {
                get_parameter(ParameterId::ChademoCurrent).value *
                        get_parameter(ParameterId::ObcDcv).value * 0.001
            } else {
                get_parameter(ParameterId::ObcDcc).value *
                        get_parameter(ParameterId::ObcDcv).value * 0.001
            },
            hw.millis());

        self.timeout_parameters(hw);
    }

    fn update_view(&mut self, hw: &mut dyn HardwareInterface) {
        // Call view.on_update()
        ((views[self.current_view]).on_update)(self.update_counter == 0, self, hw);
    }

    fn send_setting_frame(&mut self, hw: &mut dyn HardwareInterface,
            frame_id: u16, setting_id: u8, old_value: u16, new_value: u16) {
        let mut data: [u8; 8] = [0; 8];
        data[0] = setting_id;
        data[1..3].copy_from_slice(&old_value.to_be_bytes());
        data[3..5].copy_from_slice(&new_value.to_be_bytes());
        hw.send_can(bxcan::Frame::new_data(
            bxcan::StandardId::new(frame_id).unwrap(),
            bxcan::Data::new(&data).unwrap()
        ));
    }

    fn update_hvac_power(&mut self, hw: &mut dyn HardwareInterface) {
        let mut wanted_output_state = false;
        if get_parameter(ParameterId::HvacCountdown).value >= 0.0 {
            get_parameter(ParameterId::HvacCountdown).set_value(
                get_parameter(ParameterId::HvacCountdown).value - self.dt_ms as f32 * 0.001,
                hw.millis());

            if get_parameter(ParameterId::AuxVoltage).value >= 13.4 {
                wanted_output_state = true;
            }
        }

        let ms_since_last_send = hw.millis() - self.last_hvac_power_can_send_millis;
        if ms_since_last_send >= 500 {
            self.last_hvac_power_can_send_millis = hw.millis();

            if wanted_output_state == false {
                self.last_hvac_power_output_wanted_off_millis = hw.millis();
            }

            let ms_since_last_hvac_power_output_wanted_off =
                hw.millis() - self.last_hvac_power_output_wanted_off_millis;
            let power_output_state = wanted_output_state == true
                && (ms_since_last_hvac_power_output_wanted_off > 5000
                    || ms_since_last_hvac_power_output_wanted_off < 0);

            // Is this connected to something?
            hw.set_digital_output(DigitalOutput::Wakeup, power_output_state);

            // This seems to be connected to the low side of a relay coil which
            // turns on the HVAC fan and the ignition signal
            hw.set_digital_output(DigitalOutput::Pwmout1, !power_output_state); // Active low

            if get_parameter(ParameterId::HvacCountdown).value > 0.0 {
                // Request ipdm to turn on the heater and pump
                self.send_setting_frame(hw, 0x570, 2, 0, 1);
            } else {
                // Request ipdm to turn off the heater and pump
                self.send_setting_frame(hw, 0x570, 2, 0, 0);
            }
        }
    }

    fn update_charge_config(&mut self, hw: &mut dyn HardwareInterface) {
        if hw.millis() - self.last_charge_config_millis < 2000 {
            return;
        }
        self.last_charge_config_millis = hw.millis();

        if get_parameter(ParameterId::Ipdm1ChargeCompleteVoltageSetting).value as u16 != CHARGE_COMPLETE_VOLTAGE_SETTING_MV {
            self.send_setting_frame(hw, 0x570, 1, get_parameter(
                    ParameterId::Ipdm1ChargeCompleteVoltageSetting).value as u16 / 20,
                CHARGE_COMPLETE_VOLTAGE_SETTING_MV / 20);
        }

        let current_ac_charge_current_Ax5 = (get_parameter(
                ParameterId::Ipdm1AcChargeCurrentSetting).value * 5.0) as u16;
        let wanted_ac_charge_current_Ax5 = (get_parameter(
                ParameterId::AcChargeCurrentSetting).value * 5.0) as u16;

        if current_ac_charge_current_Ax5 != wanted_ac_charge_current_Ax5 {
            self.send_setting_frame(hw, 0x570, 0,
                current_ac_charge_current_Ax5,
                wanted_ac_charge_current_Ax5);
        }
    }

    fn update_http(&mut self, hw: &mut dyn HardwareInterface) {
        self.http_process.url.clear();
        self.http_process.url.push_str(base_url);
        for param in get_parameters() {
            if let Some(map) = &param.report_map {
                self.http_process.url.push_str(&str_format!(
                    fixedstr::str16,
                    "{}={:.*}&",
                    map.name,
                    map.decimals as usize,
                    param.value * map.scale
                ));
            }
        }

        match self.http_process.update(hw) {
            HttpUpdateStatus::Finished(response) => {
                if response.body.contains("request_hvac_on") {
                    get_parameters()[ParameterId::HvacCountdown as usize].set_value(180.0,
                            hw.millis());
                }
            }
            _ => {}
        }
    }

    pub fn on_button_event(&mut self, event: ButtonEvent, hw: &mut dyn HardwareInterface) {
        info!("Button event: {:?}", event);
        if ((views[self.current_view]).on_button)(event, self, hw) {
            ((views[self.current_view]).on_update)(true, self, hw);
            return;
        }
        match event {
            ButtonEvent::ButtonPress(Button::Button1 | Button::Button2 | Button::Button3) => {}
            ButtonEvent::ButtonPress(Button::Button4) => {
                if self.current_view > 0 {
                    self.current_view -= 1;
                } else {
                    self.current_view = views.len() - 1;
                }
                ((views[self.current_view]).on_update)(true, self, hw);
            }
            ButtonEvent::ButtonPress(Button::Button5) => {
                if self.current_view < views.len() - 1 {
                    self.current_view += 1;
                } else {
                    self.current_view = 0;
                }
                ((views[self.current_view]).on_update)(true, self, hw);
            }
        }
    }

    pub fn on_console_command(&mut self, command: &str, hw: &mut dyn HardwareInterface) -> bool {
        if command == "reboot" {
            hw.reboot();
            true
        } else if command == "dfu" {
            hw.activate_dfu();
            true
        } else if command == "panic" {
            panic!();
            true
        } else if command == "log can" {
            self.log_can = !self.log_can;
            info!(
                "Can logging {}",
                if self.log_can { "enabled" } else { "disabled" }
            );
            true
        } else {
            false
        }
    }

    pub fn list_console_commands(&self) {
        info!("  dfu  - Activate DFU mode");
        info!("  panic  - Call panic!()");
        info!("  log can  - Enable logging of CAN messages on console");
    }

    pub fn store_log_for_display(&mut self, buf: &str) {
        self.log_display.append(buf);
    }

    pub fn on_mainboard_rx(&mut self, buf: &str) {
        self.mainboard_log_display.append(buf);
    }

    pub fn on_can(&mut self, frame: bxcan::Frame) {
        if self.log_can {
            if let bxcan::Id::Standard(id) = frame.id() {
                if let Some(data) = frame.data() {
                    info!("on_can: {:?}: {:?}", id, data);
                }
            }
        }

        for (i, param) in get_parameters().iter_mut().enumerate() {
            if let Some(can_map) = &param.can_map {
                if let Some(data) = frame.data() {
                    if can_map.id == frame.id() {
                        match can_map.bits {
                            CanBitSelection::Bit(bit_i) => {
                                param.set_value((data[(bit_i as usize) / 8] & (1 << (bit_i % 8)))
                                        as f32 * can_map.scale,
                                    self.last_millis);
                            }
                            CanBitSelection::Uint8(byte_i) => {
                                param.set_value((data[byte_i as usize] as u8) as
                                        f32 * can_map.scale,
                                    self.last_millis);
                            }
                            CanBitSelection::Int8(byte_i) => {
                                param.set_value((data[byte_i as usize] as i8) as
                                        f32 * can_map.scale,
                                    self.last_millis);
                            }
                            CanBitSelection::Function(function) => {
                                param.set_value(function(data) * can_map.scale,
                                    self.last_millis);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn switch_to_log_view(&mut self) {
        for (i, view) in views.iter().enumerate() {
            if **view == log_view {
                self.current_view = i;
                break;
            }
        }
    }
}
