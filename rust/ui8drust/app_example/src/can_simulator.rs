use common::bxcan;

#[allow(unused_imports)]
use log::{info, warn};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

pub struct CanSimulator {
    pub txbuf: ConstGenericRingBuffer<bxcan::Frame, 10>,
    i: u64,
}

impl CanSimulator {
    pub fn new() -> Self {
        Self {
            txbuf: ConstGenericRingBuffer::new(),
            i: 0,
        }
    }

    pub fn update(&mut self, millis: u64) {
        // You can generate these using util/generate_can_simulator_txframe.py

        if self.i % 10 == 0 {
            // Outlander heater
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x398).unwrap(),
                bxcan::Data::new(b"\x01\x00\x00\x5C\x57\x00\x00\x00").unwrap(),
            ));
        } else if self.i % 10 == 1 {
            // BMS (includes cell voltages and temperatures)
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x031).unwrap(),
                bxcan::Data::new(b"\x19\x91\xA1\x06\x0A\x00\x00\x08").unwrap(),
            ));
        } else if self.i % 10 == 2 {
            // BMS (includes SoC)
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x032).unwrap(),
                bxcan::Data::new(b"\x0C\xC6\x01\xFC\x0D\xAC\xDF\x00").unwrap(),
            ));
        } else if self.i % 10 == 3 {
            // BMS (includes main contactor status)
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x030).unwrap(),
                bxcan::Data::new(b"\x07\x0C\x1D\x00\x00\x00\x00\x00").unwrap(),
            ));
        } else if self.i % 10 == 4 {
            // BMS (includes main contactor status)
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x389).unwrap(),
                bxcan::Data::new(b"\x9B\x00\x00\x30\x2C\x40\x00\x00").unwrap(),
            ));
        } else if self.i % 10 == 5 {
            // ipdm1 (includes pm state and pm contactor reason)
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x550).unwrap(),
                bxcan::Data::new(b"\x41\x00\x00\x32\xCE\x15\x00\xE6").unwrap(),
            ));
        }

        self.i += 1;
    }
}
