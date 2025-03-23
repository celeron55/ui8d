use crate::{HttpFailReason, HttpResponse, HttpUpdateStatus};

use arrayvec::ArrayString;
use fixedstr::str_format;
#[allow(unused_imports)]
use log::{info, warn};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use safe_regex::regex;

const TXBUF_SIZE: usize = 500;
const RXBUF_SIZE: usize = 500;
const URL_SIZE: usize = 500;

struct RequestStep<'a> {
    command: &'a str,
    timeout_ms: u64,
    max_retry_count: usize,
    accept_response: &'a [&'a str],
    send_command:
        fn(step: &RequestStep, request: &mut RequestStatus, driver: &mut Sim7600DriverBuffers),
    on_parse: fn(
        step: &RequestStep,
        request: &mut RequestStatus,
        driver: &mut Sim7600DriverBuffers,
    ) -> HttpUpdateStatus,
    on_timeout: fn(
        step: &RequestStep,
        request: &mut RequestStatus,
        driver: &mut Sim7600DriverBuffers,
    ) -> HttpUpdateStatus,
}

fn default_send_command(
    step: &RequestStep,
    request: &mut RequestStatus,
    driver: &mut Sim7600DriverBuffers,
) {
    driver.send_command(step.command);
}

fn contains_response(rxbuf: &ArrayString<RXBUF_SIZE>, alternate_responses: &[&str]) -> bool {
    /*if rxbuf.len() > 0 {
        info!("SIM7600: rxbuf: {:?}", rxbuf);
    }*/
    for response in alternate_responses {
        if rxbuf.contains(response) {
            info!(
                "SIM7600: Response {:?} found in rxbuf {:?}",
                response, rxbuf
            );
            return true;
        }
    }
    false
}

fn default_on_parse(
    step: &RequestStep,
    request: &mut RequestStatus,
    driver: &mut Sim7600DriverBuffers,
) -> HttpUpdateStatus {
    if contains_response(&driver.rxbuf, step.accept_response) {
        driver.rxbuf.clear();
        request.next_step(driver.millis);
        request.send_step_command(driver);
        HttpUpdateStatus::Processing
    } else {
        HttpUpdateStatus::Processing
    }
}

fn default_on_timeout(
    step: &RequestStep,
    request: &mut RequestStatus,
    driver: &mut Sim7600DriverBuffers,
) -> HttpUpdateStatus {
    if request.try_counter >= step.max_retry_count {
        // No retries left; fail
        request.step_i = None;
        driver.rxbuf.clear();
        HttpUpdateStatus::Failed(HttpFailReason::InternalTimeout)
    } else {
        // Retry
        driver.rxbuf.clear();
        request.try_timestamp = driver.millis;
        request.try_counter += 1;
        info!(
            "SIM7600: Sending step {:?} command {:?} (try {})",
            request.step_i, step.command, request.try_counter
        );
        (step.send_command)(step, request, driver);
        HttpUpdateStatus::Processing
    }
}

fn parse_byte_slice_as_u32(bytes: &[u8]) -> Option<u32> {
    // This is the stupidest thing ever
    let mut s: ArrayString<16> = ArrayString::new();
    for b in bytes {
        s.push(*b as char);
    }
    if let Ok(v) = s.parse::<u32>() {
        Some(v)
    } else {
        None
    }
}

// SIM7500_SIM7600_Series_HTTP(S)_Application_Note_V2.00.pdf

static REQUEST_STEPS: [RequestStep; 15] = [
    RequestStep {
        command: "AT+CPIN?\r",
        timeout_ms: 2000,
        max_retry_count: 30,
        accept_response: &["+CPIN: READY"],
        send_command: default_send_command,
        on_parse: default_on_parse,
        on_timeout: default_on_timeout,
    },
    RequestStep {
        command: "AT+HTTPTERM\r",
        timeout_ms: 1000,
        max_retry_count: 5,
        accept_response: &["OK\r\n", "ERROR\r\n"],
        send_command: default_send_command,
        on_parse: default_on_parse,
        on_timeout: default_on_timeout,
    },
    RequestStep {
        command: "AT+CSQ\r",
        timeout_ms: 1000,
        max_retry_count: 5,
        accept_response: &["OK\r\n"],
        send_command: default_send_command,
        on_parse: default_on_parse,
        on_timeout: default_on_timeout,
    },
    RequestStep {
        command: "AT+CGREG?\r",
        timeout_ms: 1000,
        max_retry_count: 120,
        accept_response: &[],
        send_command: default_send_command,
        on_parse: |step: &RequestStep,
                   request: &mut RequestStatus,
                   driver: &mut Sim7600DriverBuffers|
         -> HttpUpdateStatus {
            if *driver.rxbuf == *"AT+CGREG?\r\r\n+CGREG: 0,1\r\n\r\nOK\r\n"
                || *driver.rxbuf == *"AT+CGREG?\r\r\n+CGREG: 0,5\r\n\r\nOK\r\n"
            {
                // 0,1 = roaming, 0,5 = home network
                info!("{:?} response {:?} ok", step.command, driver.rxbuf);
                driver.rxbuf.clear();
                request.next_step(driver.millis);
                request.send_step_command(driver);
                HttpUpdateStatus::Processing
            } else if *driver.rxbuf == *"AT+CGREG?\r\r\n+CGREG: 0,2\r\n\r\nOK\r\n" {
                // 0,2 = not registered to any network
                // The modem is probably trying to, but the signal isn't very
                // good
                HttpUpdateStatus::Processing
            } else if driver.rxbuf.len() > 0 {
                // The result is something we don't like. Bail out if we have
                // retried many times already
                if request.try_counter > 5 {
                    request.step_i = None;
                    HttpUpdateStatus::Failed(HttpFailReason::InternalTimeout)
                } else {
                    HttpUpdateStatus::Processing
                }
            } else {
                HttpUpdateStatus::Processing
            }
        },
        on_timeout: default_on_timeout,
    },
    RequestStep {
        command: "AT+COPS?\r",
        timeout_ms: 1000,
        max_retry_count: 5,
        accept_response: &["OK\r\n"],
        send_command: default_send_command,
        on_parse: default_on_parse,
        on_timeout: default_on_timeout,
    },
    RequestStep {
        command: "AT+CGACT=0,1\r",
        timeout_ms: 1000,
        max_retry_count: 5,
        accept_response: &["OK\r\n", "ERROR\r\n"],
        send_command: default_send_command,
        on_parse: default_on_parse,
        on_timeout: default_on_timeout,
    },
    RequestStep {
        command: "AT+CGACT?\r",
        timeout_ms: 1000,
        max_retry_count: 5,
        accept_response: &["OK\r\n"],
        send_command: default_send_command,
        on_parse: default_on_parse,
        on_timeout: default_on_timeout,
    },
    RequestStep {
        command: "AT+HTTPINIT\r",
        timeout_ms: 1000,
        max_retry_count: 5,
        accept_response: &["OK\r\n", "ERROR\r\n"],
        send_command: default_send_command,
        on_parse: default_on_parse,
        on_timeout: default_on_timeout,
    },
    RequestStep {
        command: "(AT+HTTPPARA=\"CONNECTTO\")",
        timeout_ms: 3000,
        max_retry_count: 5,
        accept_response: &["OK\r\n"],
        send_command: |_step: &RequestStep,
                       request: &mut RequestStatus,
                       driver: &mut Sim7600DriverBuffers| {
            driver.send_command("AT+HTTPPARA=\"CONNECTTO\",20\r");
        },
        on_parse: default_on_parse,
        on_timeout: default_on_timeout,
    },
    RequestStep {
        command: "(AT+HTTPPARA=\"RECVTO\")",
        timeout_ms: 3000,
        max_retry_count: 5,
        accept_response: &["OK\r\n"],
        send_command: |_step: &RequestStep,
                       request: &mut RequestStatus,
                       driver: &mut Sim7600DriverBuffers| {
            driver.send_command("AT+HTTPPARA=\"RECVTO\",10\r");
        },
        on_parse: default_on_parse,
        on_timeout: default_on_timeout,
    },
    RequestStep {
        command: "(AT+HTTPPARA=\"RESPTO\")",
        timeout_ms: 3000,
        max_retry_count: 5,
        accept_response: &["OK\r\n"],
        send_command: |_step: &RequestStep,
                       request: &mut RequestStatus,
                       driver: &mut Sim7600DriverBuffers| {
            driver.send_command("AT+HTTPPARA=\"RESPTO\",20\r");
        },
        on_parse: default_on_parse,
        on_timeout: default_on_timeout,
    },
    RequestStep {
        command: "(AT+HTTPPARA=\"URL\")",
        timeout_ms: 3000,
        max_retry_count: 2,
        accept_response: &["OK\r\n"],
        send_command: |_step: &RequestStep,
                       request: &mut RequestStatus,
                       driver: &mut Sim7600DriverBuffers| {
            driver.send_command("AT+HTTPPARA=\"URL\",\"");
            driver.send_command(&request.url);
            driver.send_command("\"\r");
        },
        on_parse: default_on_parse,
        on_timeout: |step: &RequestStep,
                request: &mut RequestStatus,
                driver: &mut Sim7600DriverBuffers|
         -> HttpUpdateStatus {
            // We're not always able to read the response correctly all the way
            // until "OK\r\n" so we'll just assume it goes well if we don't see
            // the correct response
            driver.rxbuf.clear();
            request.next_step(driver.millis);
            request.send_step_command(driver);
            HttpUpdateStatus::Processing
        },
    },
    RequestStep {
        command: "AT+HTTPACTION=0\r",
        // SIM7600 manual says 120s maximum, but in reality nothing ever happens
        // after 20s anyway
        // We'll poll the result using AT+HTTPREAD? in the next step
        timeout_ms: 5000,
        max_retry_count: 1,
        accept_response: &["HTTPACTION:"],
        send_command: default_send_command,
        on_parse: |_step: &RequestStep,
                   request: &mut RequestStatus,
                   driver: &mut Sim7600DriverBuffers|
         -> HttpUpdateStatus {
            // Response format:
            // 'AT+HTTPACTION=0\r\r\n+HTTPACTION: 0,200,8\r\n'
            // 'AT+HTTPACTION=0\r\r\n+HTTPACTION:\s*\d+,\s*(\d+),\s*(\d+)', where:
            // int(match.group(1)) = status_code
            // int(match.group(2)) = content_length
            let matcher = regex!(br".*HTTPACTION:[^0-9]*[0-9]+,[^0-9]*([0-9]+),[^0-9]*([0-9]+)\r\n");

            let m = matcher.match_slices(driver.rxbuf.as_bytes());
            if let Some(m) = m {
                let (status_code_s, content_length_s) = m;
                let status_code_o = parse_byte_slice_as_u32(status_code_s);
                let content_length_o = parse_byte_slice_as_u32(content_length_s);
                if status_code_o.is_some() && content_length_o.is_some() {
                    request.status_code = status_code_o.unwrap() as u16;
                    request.content_length = content_length_o.unwrap() as usize;
                    info!(
                        "SIM7600: Parsed status_code = {} and content_length = {}",
                        request.status_code, request.content_length
                    );

                    driver.rxbuf.clear();
                    request.next_step(driver.millis);
                    request.send_step_command(driver);
                    return HttpUpdateStatus::Processing
                }
            }

            if driver.rxbuf.contains("HTTP_NONET_EVENT") {
                return HttpUpdateStatus::Failed(HttpFailReason::ServerTimeout);
            }

            HttpUpdateStatus::Processing
        },
        on_timeout: |step: &RequestStep,
                request: &mut RequestStatus,
                driver: &mut Sim7600DriverBuffers|
         -> HttpUpdateStatus {
            driver.rxbuf.clear();
            request.next_step(driver.millis);
            request.send_step_command(driver);
            HttpUpdateStatus::Processing
        },
    },
    RequestStep {
        command: "(AT+HTTPREAD?)",
        timeout_ms: 2000,
        max_retry_count: 10,
        accept_response: &[],
        send_command: |_step: &RequestStep,
                       request: &mut RequestStatus,
                       driver: &mut Sim7600DriverBuffers| {
            driver.send_command("AT+HTTPREAD?\r");
        },
        on_parse: |_step: &RequestStep,
                   request: &mut RequestStatus,
                   driver: &mut Sim7600DriverBuffers|
         -> HttpUpdateStatus {
            // Response format:
            // "AT+HTTPREAD?\r\r\n+HTTPREAD: LEN,<len>\r\n\r\nOK"

            let matcher = regex!(br".*HTTPREAD: LEN,([0-9]+)\r.*");

            let m = matcher.match_slices(driver.rxbuf.as_bytes());
            if let Some(m) = m {
                let content_length_s = m.0;
                let content_length_o = parse_byte_slice_as_u32(content_length_s);
                if content_length_o.is_some() {
                    request.content_length = content_length_o.unwrap() as usize;

                    // Only proceed if length != 0 (length will be 0 while
                    // waiting for the server to respond)
                    if request.content_length != 0 {
                        info!(
                            "SIM7600: Parsed content_length = {}",
                            request.content_length
                        );

                        driver.rxbuf.clear();
                        request.next_step(driver.millis);
                        request.send_step_command(driver);
                        return HttpUpdateStatus::Processing
                    }
                }
            }

            HttpUpdateStatus::Processing
        },
        on_timeout: default_on_timeout,
    },
    RequestStep {
        command: "(AT+HTTPREAD)",
        timeout_ms: 2000,
        max_retry_count: 1,
        accept_response: &[],
        send_command: |_step: &RequestStep,
                       request: &mut RequestStatus,
                       driver: &mut Sim7600DriverBuffers| {
            // Get content_length from AT+HTTPACTION=0 step and don't try to
            // read more than that
            let read_len = request.content_length.min(100);
            driver.send_command(&str_format!(
                fixedstr::str32,
                "AT+HTTPREAD=0,{}\r",
                read_len
            ));
        },
        on_parse: |_step: &RequestStep,
                   request: &mut RequestStatus,
                   driver: &mut Sim7600DriverBuffers|
         -> HttpUpdateStatus {
            // Get content_length from AT+HTTPACTION=0 step and don't try to
            // read more than that
            let read_len = request.content_length.min(100);
            let required_header = str_format!(
                fixedstr::str64,
                "AT+HTTPREAD=0,{}\r\r\nOK\r\n\r\n+HTTPREAD: DATA,{}\r\n",
                read_len,
                read_len
            );

            let header_pos_option = driver.rxbuf.find(&*required_header);

            if header_pos_option == None {
                // Waiting for header
                info!("SIM7600: AT+HTTPREAD: Waiting for header");
                return HttpUpdateStatus::Processing;
            }

            let header_pos = header_pos_option.unwrap();

            let data_end = header_pos + required_header.len() + read_len;

            if driver.rxbuf.len() < data_end {
                // Waiting for header
                info!("SIM7600: AT+HTTPREAD: Waiting for more data");
                return HttpUpdateStatus::Processing;
            }

            let body = &driver.rxbuf[header_pos + required_header.len()..data_end];
            let body: ArrayString<1000> = ArrayString::from(body).unwrap();
            info!("SIM7600: body: {:?}", body);

            driver.rxbuf.clear();
            request.next_step(driver.millis);

            let response = HttpResponse {
                status_code: request.status_code,
                body: body,
            };
            HttpUpdateStatus::Finished(response)
        },
        on_timeout: default_on_timeout,
    },
];

struct RequestStatus {
    request_timestamp: u64,
    url: ArrayString<URL_SIZE>,
    step_i: Option<usize>,
    step_timestamp: u64,
    try_counter: usize,
    try_timestamp: u64,
    status_code: u16,
    content_length: usize,
}

impl RequestStatus {
    fn set_step(&mut self, millis: u64, new_step_i: Option<usize>) {
        if let Some(new_step_i) = new_step_i {
            let new_step = &REQUEST_STEPS[new_step_i];
            info!(
                "SIM7600: RequestStatus: step_i {:?} -> {:?} {:?}",
                self.step_i, new_step_i, new_step.command
            );
        } else {
            info!(
                "SIM7600: RequestStatus: step_i {:?} -> {:?}",
                self.step_i, new_step_i
            );
        }
        self.step_i = new_step_i;
        self.step_timestamp = millis;
        self.try_counter = 0;
        self.try_timestamp = millis;
    }

    fn next_step(&mut self, millis: u64) {
        if self.step_i.unwrap() + 1 >= REQUEST_STEPS.len() {
            self.set_step(millis, None);
        } else {
            self.set_step(millis, Some(self.step_i.unwrap() + 1));
        }
    }

    fn send_step_command(&mut self, driver: &mut Sim7600DriverBuffers) {
        if let Some(step_i) = self.step_i {
            self.try_counter += 1;
            let step = &REQUEST_STEPS[step_i];
            info!(
                "SIM7600: Sending step {} command {:?} (try {})",
                step_i, step.command, self.try_counter
            );
            (step.send_command)(step, self, driver);
        } else {
            info!("SIM7600: Cannot send command: No steps remaining");
        }
    }
}

pub struct Sim7600DriverBuffers {
    millis: u64,
    pub txbuf: ConstGenericRingBuffer<u8, TXBUF_SIZE>,
    rxbuf: ArrayString<RXBUF_SIZE>,
}

impl Sim7600DriverBuffers {
    fn send_command(&mut self, command: &str) {
        info!("SIM7600: Command: {:?}", command);
        for c in command.bytes() {
            self.txbuf.push(c);
        }
    }
}

pub struct Sim7600Driver {
    pub buffers: Sim7600DriverBuffers,
    request: Option<RequestStatus>,
}

impl Sim7600Driver {
    pub fn new() -> Self {
        Self {
            buffers: Sim7600DriverBuffers {
                millis: 0,
                txbuf: ConstGenericRingBuffer::new(),
                rxbuf: ArrayString::new(),
            },
            request: None,
        }
    }

    pub fn update_time(&mut self, millis: u64) {
        self.buffers.millis = millis;
    }

    pub fn push(&mut self, b: u8) {
        self.buffers.rxbuf.push(b as char);
    }

    fn send_command(&mut self, command: &str) {
        self.buffers.send_command(command);
    }

    pub fn http_get_start(&mut self, url: &str) {
        info!("SIM7600: http_get_start(): url={:?}", url);
        self.buffers.rxbuf.clear();
        self.request = Some(RequestStatus {
            request_timestamp: self.buffers.millis,
            url: ArrayString::from(url).unwrap(),
            step_i: Some(0),
            step_timestamp: self.buffers.millis,
            try_counter: 0,
            try_timestamp: self.buffers.millis,
            status_code: 0,
            content_length: 0,
        });
        if let Some(request) = &mut self.request {
            request.send_step_command(&mut self.buffers);
        }
    }

    pub fn http_get_update(&mut self) -> HttpUpdateStatus {
        if let Some(request) = &mut self.request {
            if let Some(step_i) = request.step_i {
                let step = &REQUEST_STEPS[step_i];
                // On timeout, retry or fail
                if self.buffers.millis - request.try_timestamp >= step.timeout_ms {
                    info!(
                        "SIM7600: Step {}={:?} timed out. rxbuf: {:?}",
                        step_i, step.command, self.buffers.rxbuf
                    );
                    (step.on_timeout)(step, request, &mut self.buffers)
                } else {
                    // Reading, validating and parsing response
                    (step.on_parse)(step, request, &mut self.buffers)
                }
            } else {
                // We end up here after some timeouts and whatnot, so we'll just
                // consider this the same as self.request == None
                HttpUpdateStatus::NotStarted
            }
        } else {
            HttpUpdateStatus::NotStarted
        }
    }

    pub fn http_get_stop(&mut self) {
        info!("SIM7600: http_get_stop()");
        self.request = None;
    }
}
