use common::command_accumulator::CommandAccumulator;
use common::{HttpResponse, HttpUpdateStatus};

use arrayvec::ArrayString;
use fixedstr::str_format;
#[allow(unused_imports)]
use log::{info, warn};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

const TXBUF_SIZE: usize = 500;
const RXBUF_SIZE: usize = 200;

pub struct Sim7600Simulator {
    // Data sent by the simulated SIM7600
    txbuf: ConstGenericRingBuffer<u8, TXBUF_SIZE>,
    // Data received by the simulated SIM7600
    rxbuf: CommandAccumulator<RXBUF_SIZE>,
    // URL parameter
    url: ArrayString<200>,
    // Response,
    http_response: Option<HttpResponse>,
}

impl Sim7600Simulator {
    pub fn new() -> Self {
        Self {
            txbuf: ConstGenericRingBuffer::new(),
            rxbuf: CommandAccumulator::new(),
            url: ArrayString::new(),
            http_response: None,
        }
    }

    fn respond(&mut self, response: &str) {
        info!("Sim7600Simulator: Response: {:?}", response);
        for b in response.bytes() {
            self.txbuf.push(b);
        }
    }

    pub fn push(&mut self, b: u8) {
        if let Some(command) = self.rxbuf.put(b as char) {
            info!("Sim7600Simulator received command: {:?}", command);

            if *command == *"AT+CPIN?" {
                self.respond("AT+CPIN?\r\r\n+CPIN: READY\r\n\r\nOK\r\n");
            } else if *command == *"AT+CSQ" {
                self.respond("AT+CSQ\r\r\n+CSQ: 24,99\r\n\r\nOK\r\n");
            } else if *command == *"AT+CGREG?" {
                self.respond("AT+CGREG?\r\r\n+CGREG: 0,1\r\n\r\nOK\r\n");
            } else if *command == *"AT+COPS?" {
                self.respond("AT+COPS?\r\r\n+COPS: 0,0,\"elisa elisa\",7\r\n\r\nOK\r\n");
            } else if *command == *"AT+CGACT=0,1" {
                self.respond("AT+CGACT=0,1\r\r\nERROR\r\n");
            } else if *command == *"AT+CGACT?" {
                self.respond(
                    "AT+CGACT?\r\r\n+CGACT: 1,1\r\n+CGACT: 2,0\r\n+CGACT: 3,0\r\n\r\nOK\r\n",
                );
            } else if *command == *"AT+HTTPTERM" {
                self.http_response = None;
                self.respond("OK\r\n");
            } else if *command == *"AT+HTTPINIT" {
                self.respond("AT+HTTPINIT\r\r\nOK\r\n");
            } else if command.starts_with("AT+HTTPPARA=\"URL\",\"") {
                // 'AT+HTTPPARA="URL","'+url+'"\r\n'
                let url = &command[19..command.len() - 1];
                self.url.clear();
                self.url.push_str(url);
                info!("Sim7600Simulator: Parsed URL: {:?}", url);
                self.respond("OK\r\n");
            } else if *command == *"AT+HTTPACTION=0" {
                // Execute the actual HTTP request here
                let r = reqwest::blocking::get(&*self.url).unwrap();
                let status_code = r.status().as_u16();
                let mut body = ArrayString::from(&r.text().unwrap()).unwrap();
                self.http_response = Some(HttpResponse {
                    status_code: status_code,
                    body: body,
                });
                // Response format:
                // 'HTTPACTION:\s*\d+,\s*(\d+),\s*(\d+)', where:
                // int(match.group(1)) = status_code
                // int(match.group(2)) = content_length
                self.respond(&str_format!(
                    fixedstr::str64,
                    "HTTPACTION:asdf1337,foobar{},nakki{}\r\n",
                    status_code,
                    body.len(),
                ));
            } else if command.starts_with("AT+HTTPREAD=0,") {
                if let Some(http_response) = self.http_response {
                    // "AT+HTTPREAD=0,"+str(read_len)
                    let read_len_s = &command[14..command.len()];
                    info!("Sim7600Simulator: read_len_s: {:?}", read_len_s);
                    let read_len = read_len_s.parse::<usize>().unwrap();
                    // TODO: Cap length to actual content length
                    // Respond with:
                    //   'AT+HTTPREAD=0,'+str(read_len)+'\r\r\nOK\r\n\r\n+HTTPREAD: DATA,'+
                    //   str(read_len)+'\r\n'+data+'\r\n+HTTPREAD: 0\r\n'
                    self.respond(&str_format!(
                        fixedstr::str128,
                        "AT+HTTPREAD=0,{}\r\r\nOK\r\n\r\n+HTTPREAD: DATA,{}\r\n",
                        read_len,
                        read_len
                    ));
                    self.respond(&http_response.body);
                    self.respond("\r\n+HTTPREAD: 0\r\n");
                } else {
                    // Dunno what this should be
                    self.respond("ERROR\r\n");
                }
            } else {
                warn!("Sim7600Simulator: Unknown command: {:?}", command);
            }
        }
    }

    pub fn dequeue(&mut self) -> Option<u8> {
        self.txbuf.dequeue()
    }

    pub fn update(&mut self, millis: u64) {
        // TODO
    }
}
