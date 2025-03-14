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

		// Inverter
        if self.i % 20 == 0 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x1DA).unwrap(),
                bxcan::Data::new(b"\x95\x32\x18\x00\x00\x01\x02\x45").unwrap(),
            ));
        }
        if self.i % 20 == 1 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x55A).unwrap(),
                bxcan::Data::new(b"\x1a\x36\x37\x00\x5f\x00\x5b\x28").unwrap(),
            ));
        }
		// Inverter control
        if self.i % 20 == 2 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x300).unwrap(),
                bxcan::Data::new(b"\x01\x0b\xa9\x0c\x0c\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 20 == 3 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x301).unwrap(),
                bxcan::Data::new(b"\x00\x00\x71\x00\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 20 == 4 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x1D4).unwrap(),
                bxcan::Data::new(b"\x6e\x6e\x00\x00\x07\x44\x01\x28").unwrap(),
            ));
        }
        if self.i % 20 == 5 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x50B).unwrap(),
                bxcan::Data::new(b"\x00\x00\x06\xc0\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 20 == 6 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x11A).unwrap(),
                bxcan::Data::new(b"\x4e\x40\x00\xaa\xc0\x00\x00\x6b").unwrap(),
            ));
        }
		// BMS
        if self.i % 20 == 7 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x100).unwrap(),
                bxcan::Data::new(b"\x07\x0b\xa5\x00\x00\x95\x00\x00").unwrap(),
            ));
        }
        if self.i % 20 == 8 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x101).unwrap(),
                bxcan::Data::new(b"\x19\xe1\x9f\x07\x0b\x00\x0b\x0b").unwrap(),
            ));
        }
        if self.i % 20 == 9 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x102).unwrap(),
                bxcan::Data::new(b"\x0b\xcc\x02\x58\x0b\xb8\xe4\x00").unwrap(),
            ));
        }
        if self.i % 20 == 10 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x104).unwrap(),
                bxcan::Data::new(b"\x10\x04\x00\x00\x00\x00\x00\x00").unwrap(),
            ));
        }
		// PDM
        if self.i % 20 == 11 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x200).unwrap(),
                bxcan::Data::new(b"\x21\x0b\xa4\x00\x00\x0e\x00\x00").unwrap(),
            ));
        }
        if self.i % 20 == 12 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x201).unwrap(),
                bxcan::Data::new(b"\x00\x5a\x00\x00\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 20 == 13 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x202).unwrap(),
                bxcan::Data::new(b"\x08\x00\x00\x6a\x1e\x00\x3d\x00").unwrap(),
            ));
        }
        if self.i % 20 == 14 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x203).unwrap(),
                bxcan::Data::new(b"\x20\x03\xed\xf0\x00\x03\x00\x50").unwrap(),
            ));
        }
		// Coolant control
        if self.i % 20 == 15 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x400).unwrap(),
                bxcan::Data::new(b"\x0c\x3b\x00\x00\x1e\x0b\x1a\x1f").unwrap(),
            ));
        }
		// CCS controller
        if self.i % 20 == 16 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x506).unwrap(),
                bxcan::Data::new(b"\x01\x00\x00\x00\x00\x19\x00\x00").unwrap(),
            ));
        }
		// Outlander PHEV heater
        if self.i % 20 == 17 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x398).unwrap(),
                bxcan::Data::new(b"\x01\x1a\x11\x5a\x5b\x28\x00\x00").unwrap(),
            ));
        }
		// Outlander PHEV heater request
        if self.i % 20 == 18 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x188).unwrap(),
                bxcan::Data::new(b"\x03\x50\x32\x4d\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 20 == 19 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x285).unwrap(),
                bxcan::Data::new(b"\x00\x00\x14\x21\x90\xfe\x0c\x10").unwrap(),
            ));
        }

        self.i += 1;
    }
}
