use arrayvec::ArrayString;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

pub const NUM_LINES: usize = (240 - 24 * 2) / 10;
pub const LINE_MAX_LENGTH: usize = 53;

pub type LogLine = ArrayString<LINE_MAX_LENGTH>;

pub struct LogDisplay {
    pub lines: ConstGenericRingBuffer<LogLine, NUM_LINES>,
}

impl LogDisplay {
    pub fn new() -> Self {
        Self {
            lines: ConstGenericRingBuffer::new(),
        }
    }

    pub fn append(&mut self, buf: &str) {
        let mut line: ArrayString<LINE_MAX_LENGTH> = ArrayString::new();
        for c in buf.bytes() {
            if c == b'\n' || c == b'\r' {
                if !line.is_empty() {
                    self.lines.push(line);
                    line = ArrayString::new();
                }
            } else {
                if line.is_full() {
                    self.lines.push(line);
                    line = ArrayString::new();
                }
                _ = line.try_push(c as char);
            }
        }
        if !line.is_empty() {
            self.lines.push(line);
        }
    }
}
