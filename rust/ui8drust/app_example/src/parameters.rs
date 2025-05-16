use common::*;
use bxcan::{Id, StandardId};

define_parameters! {
    AuxVoltage {
        display_name: "Aux battery",
        decimals: 2,
        unit: "V",
        report_map: ReportMap { name: "vaux", decimals: 1, scale: 1.0 },
    },
    Soc {
        display_name: "SoC",
        unit: "%",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x102).unwrap()),
            bits: CanBitSelection::Uint8(6),
            scale: 100.0 / 255.0,
        },
        report_map: ReportMap { name: "er", decimals: 0, scale: 2.55 },
    },
    BatteryVMin {
        display_name: "Bat V min",
        decimals: 2,
        unit: "V",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[0] as u16) << 4) | ((data[1] as u16) >> 4)) as f32
            }),
            scale: 0.01,
        },
        report_map: ReportMap { name: "v0", decimals: 0, scale: 100.0 },
    },
    BatteryVMax {
        display_name: "Bat V max",
        decimals: 2,
        unit: "V",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                ((((data[1] & 0x0f) as u16) << 8) | data[2] as u16) as f32
            }),
            scale: 0.01,
        },
        report_map: ReportMap { name: "v1", decimals: 0, scale: 100.0 },
    },
    BatteryTMin {
        display_name: "Bat T min",
        unit: "degC",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Int8(3),
            scale: 1.0,
        },
        report_map: ReportMap { name: "t0", decimals: 0, scale: 1.0 },
    },
    BatteryTMax {
        display_name: "Bat T max",
        unit: "degC",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Int8(4),
            scale: 1.0,
        },
        report_map: ReportMap { name: "t1", decimals: 0, scale: 1.0 },
    },
    BmsMaxChargeCurrent {
        display_name: "Max charge",
        decimals: 1,
        unit: "A",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x102).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[2] as u16) << 8) | data[3] as u16) as f32
            }),
            scale: 0.1,
        },
    },
    BmsMaxDischargeCurrent {
        display_name: "Max discharge",
        decimals: 1,
        unit: "A",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x102).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[4] as u16) << 8) | data[5] as u16) as f32
            }),
            scale: 0.1,
        },
    },
    MainContactor {
        display_name: "Main contactor",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x100).unwrap()),
            bits: CanBitSelection::Bit(2),
            scale: 1.0,
        },
        report_map: ReportMap { name: "mc", decimals: 0, scale: 1.0 },
    },
    Precharging {
        display_name: "Precharging",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x100).unwrap()),
            bits: CanBitSelection::Bit(5),
            scale: 1.0,
        },
    },
    PrechargeFailed {
        display_name: "Precharge failed",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x100).unwrap()),
            bits: CanBitSelection::Bit(6),
            scale: 1.0,
        },
        report_map: ReportMap { name: "pchg_f", decimals: 0, scale: 1.0 },
    },
    Balancing {
        display_name: "Balancing",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Bit(5 * 8 + 0),
            scale: 1.0,
        },
        report_map: ReportMap { name: "b", decimals: 0, scale: 1.0 },
    },
    AllowedChargePower {
        display_name: "Chg allow",
        unit: "kW",
    },
    CcsCurrent {
        display_name: "CCS",
        unit: "A",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x506).unwrap()),
           bits: CanBitSelection::Uint8(5),
            scale: 2.0,
        },
        report_map: ReportMap { name: "ccsc", decimals: 0, scale: 1.0 },
    },
    ChademoCurrent {
        display_name: "Chademo",
        unit: "A",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x500).unwrap()),
            bits: CanBitSelection::Uint8(5),
            scale: 1.0,
        },
        report_map: ReportMap { name: "chac", decimals: 0, scale: 1.0 },
    },
    ChargePower {
        display_name: "Charge power",
        decimals: 1,
        unit: "kW",
        report_map: ReportMap { name: "chgp", decimals: 0, scale: 0.001 },
    },
    BmsChargeCompleteVoltageSetting {
        display_name: "BmsChgCompV",
        unit: "mV",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x104).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[0] as u16) << 8) | data[1] as u16) as f32
            }),
            scale: 1.0,
        },
        report_map: ReportMap { name: "bccv", decimals: 0, scale: 1.0 },
    },
    IpdmChargeCompleteVoltageSetting {
        display_name: "IpdmChgCompV",
        unit: "mV",
        report_map: ReportMap { name: "i1ccv", decimals: 0, scale: 1.0 },
    },
    AcChargeCurrentSetting {
        display_name: "AcCurSet",
        value: 10.0,
        unit: "A",
        report_map: ReportMap { name: "acc", decimals: 0, scale: 1.0 },
    },
    IpdmAcChargeCurrentSetting {
        display_name: "IpdmAcCurSet",
        unit: "A",
        report_map: ReportMap { name: "i1acc", decimals: 0, scale: 1.0 },
    },
    AcObcState {
        display_name: "AcObcSt->Focci",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::Uint8(1),
            scale: 1.0,
        },
        report_map: ReportMap { name: "aos", decimals: 0, scale: 1.0 },
    },
    FoccciCPPWM {
        display_name: "Foccci CP PWM",
        unit: "%",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x506).unwrap()),
            bits: CanBitSelection::Uint8(1),
            scale: 1.0,
        },
        report_map: ReportMap { name: "cp", decimals: 0, scale: 1.0 },
    },
    ObcEvsePwm {
        display_name: "OBC CP PWM",
        unit: "%",
        can_map: CanMap {
           id: Id::Standard(StandardId::new(0x38a).unwrap()),
            bits: CanBitSelection::Uint8(3),
            scale: 1.0,
        },
    },
    ObcChargeCurrentRequest {
        display_name: "OBC req A",
        decimals: 1,
        unit: "A",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x286).unwrap()),
            bits: CanBitSelection::Uint8(2),
            scale: 0.1,
        },
    },
    ObcChargeVoltageRequest {
        display_name: "OBC req V",
        decimals: 1,
        unit: "V",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x286).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[0] as u16) << 8) | data[1] as u16) as f32
            }),
            scale: 0.1,
        },
    },
    DcdcStatus {
        display_name: "DCDC status",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x377).unwrap()),
            bits: CanBitSelection::Uint8(7),
            scale: 1.0,
        },
    },
    DcdcAuxVoltage {
        display_name: "DCDC aux V",
        decimals: 2,
        unit: "V",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x377).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[0] as u16) << 8) | data[1] as u16) as f32
            }),
            scale: 0.01,
        },
    },
    DcdcCurrent {
        display_name: "DCDC current",
        decimals: 1,
        unit: "A",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x377).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[2] as u16) << 8) | data[3] as u16) as f32
            }),
            scale: 0.1,
        },
    },
    HeaterT {
        display_name: "Heater T",
        unit: "degC",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                let t1 = data[3] as i8 - 40;
                let t2 = data[4] as i8 - 40;
                (if t1 > t2 { t1 } else { t2 }) as f32
            }),
            scale: 1.0,
        },
        report_map: ReportMap { name: "ht", decimals: 0, scale: 1.0 },
    },
    HeaterHeating {
        display_name: "Heater heating",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                if data[5] > 0 { 1.0 } else { 0.0 }
            }),
            scale: 1.0,
        },
        report_map: ReportMap { name: "ohh", decimals: 0, scale: 1.0 },
    },
    HeaterPowerPercent {
        display_name: "Heater power",
        unit: "%",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                if data[5] > 0 { 100.0 } else { 0.0 }
            }),
            scale: 1.0,
        },
        report_map: ReportMap { name: "he", decimals: 0, scale: 1.0 },
    },
    CabinT {
        display_name: "CabinT",
        decimals: 1,
        unit: "degC",
        report_map: ReportMap { name: "cabin_t", decimals: 1, scale: 1.0 },
    },
    HvacCountdown {
        display_name: "HvacCountdown",
        decimals: 1,
        unit: "s",
    },
    ObcDcv {
        display_name: "OBC DC V",
        unit: "V",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x389).unwrap()),
            bits: CanBitSelection::Uint8(0),
            scale: 2.0,
        },
        report_map: ReportMap { name: "pv", decimals: 0, scale: 10.0 },
    },
    ObcDcc {
        display_name: "OBC DC A",
        decimals: 1,
        unit: "Adc",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x389).unwrap()),
            bits: CanBitSelection::Uint8(2),
            scale: 0.1,
        },
        report_map: ReportMap { name: "pc", decimals: 0, scale: 10.0 },
    },
    AcVoltage {
        display_name: "OBC AC V",
        unit: "V",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x389).unwrap()),
            bits: CanBitSelection::Uint8(1),
            scale: 1.0,
        },
        report_map: ReportMap { name: "ac", decimals: 0, scale: 1.0 },
    },
    Speed {
        display_name: "Speed",
        unit: "km/h",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x051).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[2] as u16) << 8) | data[3] as u16) as f32
            }),
            scale: 80.0 / 5300.0,
        },
        report_map: ReportMap { name: "speed", decimals: 0, scale: 1.0 },
    },
    CruiseRequested {
        display_name: "Cruise requested",
        unit: "",
        report_map: ReportMap { name: "crur", decimals: 0, scale: 1.0 },
    },
    CruiseActive {
        display_name: "Cruise active",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x300).unwrap()),
            bits: CanBitSelection::Bit(2),
            scale: 1.0,
        },
        report_map: ReportMap { name: "cru", decimals: 0, scale: 1.0 },
    },
    InverterT {
        display_name: "InverterT",
        unit: "degC",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x55a).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                let fahrenheit = data[2];
                (fahrenheit as f32 - 32.0) * 5.0 / 9.0
            }),
            scale: 1.0,
        },
        report_map: ReportMap { name: "it", decimals: 0, scale: 1.0 },
    },
    MotorT {
        display_name: "MotorT",
        unit: "degC",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x55a).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                let fahrenheit = data[1];
                (fahrenheit as f32 - 32.0) * 5.0 / 9.0
            }),
            scale: 1.0,
        },
        report_map: ReportMap { name: "mt", decimals: 0, scale: 1.0 },
    },
    IpdmIgnition {
        display_name: "IPDM Ignition",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Bit(6),
            scale: 1.0,
        },
    },
    IpdmReqMC {
        display_name: "IPDM req MC",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Bit(0),
            scale: 1.0,
        },
    },
    IpdmPcbT {
        display_name: "IPDM PCB T",
        unit: "degC",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Int8(5),
            scale: 1.0,
        },
    },
    IpdmGroup1OC {
        display_name: "IPDM OC 1",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Bit(56),
            scale: 1.0,
        },
    },
    IpdmGroup2OC {
        display_name: "IPDM OC 2",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Bit(57),
            scale: 1.0,
        },
    },
    IpdmGroup3OC {
        display_name: "IPDM OC 3",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Bit(58),
            scale: 1.0,
        },
    },
    IpdmGroup4OC {
        display_name: "IPDM OC 4",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x200).unwrap()),
            bits: CanBitSelection::Bit(59),
            scale: 1.0,
        },
    },
    IpdmCurrent1 {
        display_name: "IPDM group 1",
        decimals: 3,
        unit: "A",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x406).unwrap()),
            bits: CanBitSelection::BeUnsigned(0, 12),
            scale: 1.0 / 256.0,
        },
    },
    IpdmCurrent2 {
        display_name: "IPDM group 2",
        decimals: 3,
        unit: "A",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x406).unwrap()),
            bits: CanBitSelection::BeUnsigned(12, 12),
            scale: 1.0 / 256.0,
        },
    },
    IpdmCurrent3 {
        display_name: "IPDM group 3",
        decimals: 3,
        unit: "A",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x406).unwrap()),
            bits: CanBitSelection::BeUnsigned(24, 12),
            scale: 1.0 / 256.0,
        },
    },
    IpdmCurrent4 {
        display_name: "IPDM group 4",
        decimals: 3,
        unit: "A",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x406).unwrap()),
            bits: CanBitSelection::BeUnsigned(36, 12),
            scale: 1.0 / 256.0,
        },
    },
    IpdmCurrentL {
        display_name: "IPDM group L",
        decimals: 2,
        unit: "A",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x406).unwrap()),
            bits: CanBitSelection::BeUnsigned(48, 12),
            scale: 1.0 / 256.0,
        },
    },
    IpdmM1 {
        display_name: "IPDM M1",
        decimals: 3,
        unit: "V",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::BeUnsigned(16, 12),
            scale: 1.0 / 128.0,
        },
    },
    IpdmM2 {
        display_name: "IPDM M2",
        decimals: 3,
        unit: "V",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::BeUnsigned(28, 12),
            scale: 1.0 / 128.0,
        },
    },
    IpdmM3 {
        display_name: "IPDM M3",
        decimals: 3,
        unit: "V",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::BeUnsigned(40, 12),
            scale: 1.0 / 128.0,
        },
    },
    IpdmM4 {
        display_name: "IPDM M4",
        decimals: 3,
        unit: "V",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::BeUnsigned(52, 12),
            scale: 1.0 / 128.0,
        },
    },
    IpdmM5 {
        display_name: "IPDM M5",
        decimals: 3,
        unit: "V",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x405).unwrap()),
            bits: CanBitSelection::BeUnsigned(0, 12),
            scale: 1.0 / 128.0,
        },
    },
    IpdmM6 {
        display_name: "IPDM M6",
        decimals: 3,
        unit: "V",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x405).unwrap()),
            bits: CanBitSelection::BeUnsigned(12, 12),
            scale: 1.0 / 128.0,
        },
    },
    IpdmM7 {
        display_name: "IPDM M7",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::Bit(1),
            scale: 1.0,
        },
    },
    IpdmM8 {
        display_name: "IPDM M8",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::Bit(2),
            scale: 1.0,
        },
    },
    IpdmM9 {
        display_name: "IPDM M9",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::Bit(3),
            scale: 1.0,
        },
    },
    IpdmM10 {
        display_name: "IPDM M10",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::Bit(4),
            scale: 1.0,
        },
    },
    IpdmM11 {
        display_name: "IPDM M11",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::Bit(5),
            scale: 1.0,
        },
    },
    IpdmM12 {
        display_name: "IPDM M12",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::Bit(6),
            scale: 1.0,
        },
    },
    IpdmM13 {
        display_name: "IPDM M13",
        unit: "",
        can_map: CanMap {
            id: Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::Bit(7),
            scale: 1.0,
        },
    },
    PmState {
        display_name: "PmState",
        unit: "",
        report_map: ReportMap { name: "pms", decimals: 0, scale: 1.0 },
    },
    PmCr {
        display_name: "PmCr",
        unit: "",
        report_map: ReportMap { name: "pmcr", decimals: 0, scale: 1.0 },
    },
    RangeKm {
        display_name: "Range",
        unit: "km",
    },
    TripKm {
        display_name: "Trip",
        unit: "km",
    },
    TripEnergy {
        display_name: "Trip",
        unit: "Wh",
    },
    TripConsumption {
        display_name: "Trip",
        unit: "Wh/km",
    },
    RecentKm {
        display_name: "Recent",
        unit: "km",
    },
    RecentEnergy {
        display_name: "Recent",
        unit: "Wh",
    },
    RecentConsumption {
        display_name: "Recent",
        unit: "Wh/km",
    },
    TicksMs {
        display_name: "Ticks",
        unit: "ms",
        report_map: ReportMap { name: "t", decimals: 0, scale: 0.001 },
    },
}
