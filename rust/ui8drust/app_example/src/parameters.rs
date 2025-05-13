use common::*;

use bxcan::StandardId;
use int_enum::IntEnum;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

#[repr(usize)]
#[derive(IntEnum, Debug, Clone, Copy)]
pub enum ParameterId {
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
    CcsCurrent = 30,
    ChademoCurrent = 31,
    ChargePower = 32,
    TripEnergy = 33,
    RecentEnergy = 34,
    Speed = 35,
    CruiseActive = 36,
    CruiseRequested = 37,
    FoccciCPPWM = 38,
    AcObcState = 39,
    BmsMaxChargeCurrent = 40,
    BmsMaxDischargeCurrent = 41,
    ObcChargeCurrentRequest = 42,
    ObcChargeVoltageRequest = 43,
    ObcEvsePwm = 44,
    DcdcStatus = 45,
    DcdcAuxVoltage = 46,
    DcdcCurrent = 47,
    InverterT = 48,
    MotorT = 49,
    IpdmIgnition = 50,
    IpdmReqMC = 51,
    IpdmGroup1OC = 52,
    IpdmGroup2OC = 53,
    IpdmGroup3OC = 54,
    IpdmGroup4OC = 55,
    Precharging = 56,
}

static mut PARAMETERS: [Parameter<ParameterId>; 57] = [
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
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
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
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
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
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
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
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
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
            id: bxcan::Id::Standard(StandardId::new(0x102).unwrap()),
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
            id: bxcan::Id::Standard(StandardId::new(0x100).unwrap()),
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
            id: bxcan::Id::Standard(StandardId::new(0x100).unwrap()),
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
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
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
        display_name: "OBC DC V",
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
        display_name: "OBC DC A",
        value: f32::NAN,
        decimals: 1,
        unit: "Adc",
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
        display_name: "OBC AC V",
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
        can_map: None,
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
        can_map: None,
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
            id: bxcan::Id::Standard(StandardId::new(0x104).unwrap()),
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
        can_map: None,
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
        can_map: None,
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
    Parameter {
        id: ParameterId::CcsCurrent,
        display_name: "CCS",
        value: f32::NAN,
        decimals: 0,
        unit: "A",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x506).unwrap()),
            bits: CanBitSelection::Uint8(5),
            scale: 2.0,
        }),
        report_map: Some(ReportMap {
            name: "ccsc",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::ChademoCurrent,
        display_name: "Chademo",
        value: f32::NAN,
        decimals: 0,
        unit: "A",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x500).unwrap()),
            bits: CanBitSelection::Uint8(5),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "chac",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::ChargePower,
        display_name: "Charge power",
        value: f32::NAN,
        decimals: 1,
        unit: "kW",
        can_map: None,
        report_map: Some(ReportMap {
            name: "chgp",
            decimals: 0,
            scale: 0.001,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::TripEnergy,
        display_name: "Trip",
        value: f32::NAN,
        decimals: 0,
        unit: "Wh",
        can_map: None,
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::RecentEnergy,
        display_name: "Recent",
        value: f32::NAN,
        decimals: 0,
        unit: "Wh",
        can_map: None,
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::Speed,
        display_name: "Speed",
        value: f32::NAN,
        decimals: 0,
        unit: "km/h",
        can_map: Some(CanMap {
            // MG2 speed scaled for 2nd gear
            id: bxcan::Id::Standard(StandardId::new(0x051).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[2] as u16) << 8) | data[3] as u16) as f32
            }),
            scale: 80.0 / 5300.0,
        }),
        report_map: Some(ReportMap {
            name: "speed",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::CruiseActive,
        display_name: "Cruise active",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x300).unwrap()),
            bits: CanBitSelection::Bit(2),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "cru",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::CruiseRequested,
        display_name: "Cruise requested",
        value: 0.0,
        decimals: 0,
        unit: "",
        can_map: None,
        report_map: Some(ReportMap {
            name: "crur",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::FoccciCPPWM,
        display_name: "Foccci CP PWM",
        value: f32::NAN,
        decimals: 0,
        unit: "%",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x506).unwrap()),
            bits: CanBitSelection::Uint8(1),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "cp",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        // Published by ipdm56 for usage by Foccci
        // Foccci dictates this value as:
        // {IDLE=0, LOCK=1, CHARGE=2, PAUSE=3, COMPLETE=4, ERROR=5}
        id: ParameterId::AcObcState,
        display_name: "AcObcSt->Focci",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::Uint8(1),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "aos",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BmsMaxChargeCurrent,
        display_name: "Max charge",
        value: f32::NAN,
        decimals: 1,
        unit: "A",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x102).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[2] as u16) << 8) | data[3] as u16) as f32
            }),
            scale: 0.1,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BmsMaxDischargeCurrent,
        display_name: "Max discharge",
        value: f32::NAN,
        decimals: 1,
        unit: "A",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x102).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[4] as u16) << 8) | data[5] as u16) as f32
            }),
            scale: 0.1,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::ObcChargeCurrentRequest,
        display_name: "OBC req A",
        value: f32::NAN,
        decimals: 1,
        unit: "A",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x286).unwrap()),
            bits: CanBitSelection::Uint8(2),
            scale: 0.1,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::ObcChargeVoltageRequest,
        display_name: "OBC req V",
        value: f32::NAN,
        decimals: 1,
        unit: "V",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x286).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[0] as u16) << 8) | data[1] as u16) as f32
            }),
            scale: 0.1,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::ObcEvsePwm,
        display_name: "OBC CP PWM",
        value: f32::NAN,
        decimals: 0,
        unit: "%",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x38a).unwrap()),
            bits: CanBitSelection::Uint8(3),
            scale: 1.0,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::DcdcStatus,
        display_name: "DCDC status",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x377).unwrap()),
            bits: CanBitSelection::Uint8(7),
            scale: 1.0,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::DcdcAuxVoltage,
        display_name: "DCDC aux V",
        value: f32::NAN,
        decimals: 2,
        unit: "V",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x377).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[0] as u16) << 8) | data[1] as u16) as f32
            }),
            scale: 0.01,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::DcdcCurrent,
        display_name: "DCDC current",
        value: f32::NAN,
        decimals: 1,
        unit: "A",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x377).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[2] as u16) << 8) | data[3] as u16) as f32
            }),
            scale: 0.1,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::InverterT,
        display_name: "InverterT",
        value: f32::NAN,
        decimals: 0,
        unit: "degC",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x55a).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                let fahrenheit = data[2];
                (fahrenheit as f32 - 32.0) * 5.0 / 9.0
            }),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "it",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::MotorT,
        display_name: "MotorT",
        value: f32::NAN,
        decimals: 0,
        unit: "degC",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x55a).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                let fahrenheit = data[1];
                (fahrenheit as f32 - 32.0) * 5.0 / 9.0
            }),
            scale: 1.0,
        }),
        report_map: Some(ReportMap {
            name: "mt",
            decimals: 0,
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::IpdmIgnition,
        display_name: "IPDM Ignition",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Bit(6),
            scale: 1.0,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::IpdmReqMC,
        display_name: "IPDM req MC",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Bit(0),
            scale: 1.0,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::IpdmGroup1OC,
        display_name: "IPDM OC 1",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Bit(56),
            scale: 1.0,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::IpdmGroup2OC,
        display_name: "IPDM OC 2",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Bit(57),
            scale: 1.0,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::IpdmGroup3OC,
        display_name: "IPDM OC 3",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Bit(58),
            scale: 1.0,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::IpdmGroup4OC,
        display_name: "IPDM OC 4",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Bit(59),
            scale: 1.0,
        }),
        report_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::Precharging,
        display_name: "Precharging",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x100).unwrap()),
            bits: CanBitSelection::Bit(5),
            scale: 1.0,
        }),
        report_map: None,
        update_timestamp: 0,
    },
];

pub fn get_parameters() -> &'static mut [Parameter<'static, ParameterId>] {
    unsafe {
        return &mut PARAMETERS;
    }
}
pub fn get_parameter(id: ParameterId) -> &'static mut Parameter<'static, ParameterId> {
    unsafe {
        return &mut PARAMETERS[usize::from(id)];
    }
}

pub fn check_parameter_id_consistency() -> bool {
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

