#![no_std]

// You need to supply this at build time
// Example: "http://example.com/report?id=test&"
const base_url: &str = env!("BASE_URL");

const CHARGE_COMPLETE_VOLTAGE_SETTING_MV: u16 = 4160; // Should be divisible by 20

use common::*;

pub mod can_simulator;

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

#[repr(usize)]
#[derive(IntEnum, Debug, Clone, Copy)]
enum ParameterId {
    TicksMs = 0,
    AuxVoltage = 1,
    BatteryTMin = 2,
    BatteryTMax = 3,
    BatteryVMin = 4,
    BatteryVMax = 5,
    Soc = 6,
    RangeKm = 7,
    AllowedChargePower = 8,
    TripKm = 9,
    TripConsumption = 10,
    RecentKm = 11,
    RecentConsumption = 12,
    HvacCountdown = 13,
    HeaterT = 14,
    HeaterHeating = 15,
    HeaterPowerPercent = 16,
    CabinT = 17,
    MainContactor = 18,
    PrechargeFailed = 19,
    Balancing = 20,
    ObcDcv = 21,
    ObcDcc = 22,
    AcVoltage = 23,
    PmState = 24,
    PmCr = 25,
    BmsChargeCompleteVoltageSetting = 26,
    Ipdm1ChargeCompleteVoltageSetting = 27,
    Ipdm1AcChargeCurrentSetting = 28,
    AcChargeCurrentSetting = 29,
}

static mut PARAMETERS: [Parameter<ParameterId>; 30] = [
    Parameter {
        id: ParameterId::TicksMs,
        display_name: "Ticks",
        value: 0.0,
        decimals: 0,
        unit: "ms",
        can_map: None,
        report_map: Some(ReportMap {
            name: "t",
            decimals: 0,
            scale: 0.001,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::AuxVoltage,
        display_name: "Aux battery",
        value: f32::NAN,
        decimals: 2,
        unit: "V",
        can_map: None,
        report_map: Some(ReportMap {
            name: "vaux",
            decimals: 1,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BatteryTMin,
        display_name: "Bat T min",
        value: f32::NAN,
        decimals: 0,
        unit: "degC",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x031).unwrap()),
            bits: CanBitSelection::Int8(3),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "t0",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BatteryTMax,
        display_name: "Bat T max",
        value: f32::NAN,
        decimals: 0,
        unit: "degC",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x031).unwrap()),
            bits: CanBitSelection::Int8(4),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "t1",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BatteryVMin,
        display_name: "Bat V min",
        value: f32::NAN,
        decimals: 2,
        unit: "V",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x031).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[0] as u16) << 4) | ((data[1] as u16) >> 4)) as f32
            }),
            scale: 0.01,
        }),
        report_map: Some(ReportMap {
            name: "v0",
            decimals: 0,
            scale: 100.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BatteryVMax,
        display_name: "Bat V max",
        value: f32::NAN,
        decimals: 2,
        unit: "V",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x031).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                ((((data[1] & 0x0f) as u16) << 8) | data[2] as u16) as f32
            }),
            scale: 0.01,
        }),
        report_map: Some(ReportMap {
            name: "v1",
            decimals: 0,
            scale: 100.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::Soc,
        display_name: "SoC",
        value: f32::NAN,
        decimals: 0,
        unit: "%",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x032).unwrap()),
            bits: CanBitSelection::Uint8(6),
            scale: 100.0 / 255.0,
        }),
        report_map: Some(ReportMap {
            name: "er",
            decimals: 0,
            scale: 2.55,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::RangeKm,
        display_name: "Range",
        value: f32::NAN,
        decimals: 0,
        unit: "km",
        can_map: None,
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::AllowedChargePower,
        display_name: "Chg allow",
        value: f32::NAN,
        decimals: 0,
        unit: "kW",
        can_map: None,
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::TripKm,
        display_name: "Trip",
        value: f32::NAN,
        decimals: 0,
        unit: "km",
        can_map: None,
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::TripConsumption,
        display_name: "Trip",
        value: f32::NAN,
        decimals: 0,
        unit: "Wh/km",
        can_map: None,
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::RecentKm,
        display_name: "Recent",
        value: f32::NAN,
        decimals: 0,
        unit: "km",
        can_map: None,
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::RecentConsumption,
        display_name: "Recent",
        value: f32::NAN,
        decimals: 0,
        unit: "Wh/km",
        can_map: None,
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::HvacCountdown,
        display_name: "HvacCountdown",
        value: 0.0,
        decimals: 1,
        unit: "s",
        can_map: None,
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::HeaterT,
        display_name: "Heater T",
        value: f32::NAN,
        decimals: 0,
        unit: "degC",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                let t1 = data[3] as i8 - 40;
                let t2 = data[4] as i8 - 40;
                (if t1 > t2 { t1 } else { t2 }) as f32
            }),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "ht",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::HeaterHeating,
        display_name: "Heater heating",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                if data[5] > 0 {
                    1.0
                } else {
                    0.0
                }
            }),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "ohh",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::HeaterPowerPercent,
        display_name: "Heater power",
        value: f32::NAN,
        decimals: 0,
        unit: "%",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                // TODO: This accurate. The heater can be requested different
                //       power levels in 0x188
                if data[5] > 0 {
                    100.0
                } else {
                    0.0
                }
            }),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "he",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::CabinT,
        display_name: "CabinT",
        value: f32::NAN,
        decimals: 1,
        unit: "degC",
        can_map: None,
        report_map: Some(ReportMap {
            name: "cabin_t",
            decimals: 1,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::MainContactor,
        display_name: "Main contactor",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x030).unwrap()),
            bits: CanBitSelection::Bit(2),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "mc",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::PrechargeFailed,
        display_name: "Precharge failed",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x030).unwrap()),
            bits: CanBitSelection::Bit(6),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "pchg_f",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::Balancing,
        display_name: "Balancing",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x031).unwrap()),
            bits: CanBitSelection::Bit(5 * 8 + 0),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "b",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::ObcDcv,
        display_name: "OBC V DC",
        value: f32::NAN,
        decimals: 0,
        unit: "V",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x389).unwrap()),
            bits: CanBitSelection::Uint8(0),
            scale: 2.0,
        }),
        report_map: Some(ReportMap {
            name: "pv",
            decimals: 0,
            scale: 10.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::ObcDcc,
        display_name: "OBC A DC",
        value: f32::NAN,
        decimals: 1,
        unit: "A",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x389).unwrap()),
            bits: CanBitSelection::Uint8(2),
            scale: 0.1,
        }),
        report_map: Some(ReportMap {
            name: "pc",
            decimals: 0,
            scale: 10.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::AcVoltage,
        display_name: "AC voltage",
        value: f32::NAN,
        decimals: 0,
        unit: "V",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x389).unwrap()),
            bits: CanBitSelection::Uint8(1),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "ac",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::PmState,
        display_name: "PmState",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x550).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 { (data[5] & 0x0f) as f32 }),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "pms",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::PmCr,
        display_name: "PmCr",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x550).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                ((data[5] & 0xf0) >> 4) as f32
            }),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "pmcr",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BmsChargeCompleteVoltageSetting,
        display_name: "BmsChgCompV",
        value: f32::NAN,
        decimals: 0,
        unit: "mV",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x034).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[0] as u16) << 8) | data[1] as u16) as f32
            }),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "bccv",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::Ipdm1ChargeCompleteVoltageSetting,
        display_name: "Ipdm1ChgCompV",
        value: f32::NAN,
        decimals: 0,
        unit: "mV",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x550).unwrap()),
            bits: CanBitSelection::Uint8(4),
            scale: 20.0,
        }),
        report_map: Some(ReportMap {
            name: "i1ccv",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::Ipdm1AcChargeCurrentSetting,
        display_name: "Ipdm1AcCurSet",
        value: f32::NAN,
        decimals: 0,
        unit: "A",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x550).unwrap()),
            bits: CanBitSelection::Uint8(3),
            scale: 0.2,
        }),
        report_map: Some(ReportMap {
            name: "i1acc",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::AcChargeCurrentSetting,
        display_name: "AcCurSet",
        value: 10.0,
        decimals: 0,
        unit: "A",
        can_map: None,
        report_map: Some(ReportMap {
            name: "acc",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
];

fn get_parameters() -> &'static mut [Parameter<'static, ParameterId>] {
    unsafe {
        return &mut PARAMETERS;
    }
}
fn get_parameter(id: ParameterId) -> &'static mut Parameter<'static, ParameterId> {
    unsafe {
        return &mut PARAMETERS[usize::from(id)];
    }
}

fn check_parameter_id_consistency() -> bool {
    for (i, param) in get_parameters().iter().enumerate() {
        if usize::from(param.id) != i {
            error!(
                "Parameter [{}].id == {}: ID mismatch",
                i,
                usize::from(param.id)
            );
            return false;
        }
    }
    return true;
}

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

static main_view: View = View {
    on_update: |redraw: bool, state: &mut MainState, hw: &mut dyn HardwareInterface| {
        if redraw {
            draw_brand_background(hw);
            draw_view_number(state.current_view, hw);
            draw_button_action(0, "Reboot", false, hw);
            //draw_button_action(0, "Cruis", false, hw);
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
        draw_parameter_dual(
            "OBC",
            ParameterId::ObcDcv,
            ParameterId::ObcDcc,
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
        draw_parameter(
            ParameterId::AcVoltage,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 6,
            redraw,
            hw,
        );
        draw_parameter(
            ParameterId::AuxVoltage,
            TEXT_TOP_ROW_Y + PARAM_ROW_HEIGHT * 7,
            redraw,
            hw,
        );
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
                return false;
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
        if let Ok(id) = ParameterId::try_from(i1) {
            draw_parameter(
                ParameterId::try_from(i1).unwrap(),
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

static views: [&View; 3] = [&main_view, &all_params_view, &log_view];

pub struct MainState {
    update_counter: u32,
    log_display: LogDisplay,
    current_view: usize,
    log_can: bool,
    all_params_view_page: usize,
    last_millis: u64,
    dt_ms: u64,
    last_http_request_millis: u64,
    last_hvac_power_can_send_millis: u64,
    last_hvac_power_output_wanted_off_millis: u64,
    sim7600_power_cycle_start_timestamp: u64,
    sim7600_power_cycle_error_counter: u32,
    last_charge_config_millis: u64,
}

impl MainState {
    pub fn new() -> Self {
        Self {
            update_counter: 0,
            log_display: LogDisplay::new(),
            current_view: 0,
            log_can: false,
            all_params_view_page: 0,
            last_millis: 0,
            dt_ms: 0,
            last_http_request_millis: 0,
            last_hvac_power_can_send_millis: 0,
            last_hvac_power_output_wanted_off_millis: 0,
            sim7600_power_cycle_start_timestamp: 0,
            sim7600_power_cycle_error_counter: 0,
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

        // TODO: Update ParameterId::CabinT based on ADC

        self.timeout_parameters(hw);
    }

    fn update_view(&mut self, hw: &mut dyn HardwareInterface) {
        // This check happens, and has to happen, on the first update, and if
        // the parameter ids are not consistent, the view has to be changed to
        // some view which doesn't use get_parameters() (like the log view), in
        // order to not cause a panic right away.
        if self.update_counter == 0 && !check_parameter_id_consistency() {
            error!("Parameter ID consistency error");
            self.switch_to_log_view();
        }

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
            hw.set_digital_output(DigitalOutput::Wakeup, power_output_state);

            if get_parameter(ParameterId::HvacCountdown).value > 0.0 {
                // Request ipdm1 to turn on the heater and pump
                self.send_setting_frame(hw, 0x570, 2, 0, 1);
            } else {
                // Request ipdm1 to turn off the heater and pump
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
        if hw.millis() - self.sim7600_power_cycle_start_timestamp < 3000 {
            hw.set_digital_output(DigitalOutput::Sim7600PowerInhibit, true);
        } else {
            hw.set_digital_output(DigitalOutput::Sim7600PowerInhibit, false);
        }

        if hw.millis() - self.sim7600_power_cycle_start_timestamp < 4000 {
            return;
        }

        let mut url: ArrayString<500> = ArrayString::new();
        url.push_str(base_url);

        for param in get_parameters() {
            if let Some(map) = &param.report_map {
                url.push_str(&str_format!(
                    fixedstr::str16,
                    "{}={:.*}&",
                    map.name,
                    map.decimals as usize,
                    param.value * map.scale
                ));
            }
        }

        let ms_since_last_request = hw.millis() - self.last_http_request_millis;

        match hw.http_get_update() {
            HttpUpdateStatus::NotStarted => {
                if ms_since_last_request > 10000 || ms_since_last_request < 0 {
                    info!("http_get_update() -> NotStarted; starting");
                    hw.http_get_start(&url);
                    self.last_http_request_millis = hw.millis();
                }
            }
            HttpUpdateStatus::Processing => {
                if self.update_counter % 100 == 0 {
                    info!("http_get_update() -> Processing");
                }
            }
            HttpUpdateStatus::Failed(reason) => {
                info!("http_get_update() -> Failed: {:?}", reason);
                if reason == HttpFailReason::InternalTimeout ||
                        reason == HttpFailReason::InternalError ||
                        reason == HttpFailReason::Unknown {
                    self.sim7600_power_cycle_error_counter += 1;
                    if self.sim7600_power_cycle_error_counter >= 10 {
                        info!("-!- Power cycling SIM7600");
                        self.sim7600_power_cycle_error_counter = 0;
                        self.sim7600_power_cycle_start_timestamp = hw.millis();
                    }
                } else {
                    hw.http_get_stop();
                }
            }
            HttpUpdateStatus::Finished(response) => {
                info!("http_get_update() -> Finished; response: {:?}", response);
                hw.http_get_stop();

                if response.body.contains("request_hvac_on") {
                    get_parameters()[usize::from(ParameterId::HvacCountdown)].set_value(180.0,
                            hw.millis());
                }
            }
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
