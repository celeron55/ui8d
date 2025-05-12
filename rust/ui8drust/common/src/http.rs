use crate::{HardwareInterface, DigitalOutput, HttpUpdateStatus, HttpFailReason, HttpResponse};
use arrayvec::ArrayString;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub struct HttpProcess {
    pub url: ArrayString<500>,
    update_counter: u32,
    last_http_request_millis: u64,
    sim7600_power_cycle_start_timestamp: u64,
    sim7600_power_cycle_error_counter: u32,
}

impl HttpProcess {
    pub fn new() -> Self {
        Self {
            url: ArrayString::new(),
            update_counter: 0,
            last_http_request_millis: 0,
            sim7600_power_cycle_start_timestamp: 0,
            sim7600_power_cycle_error_counter: 0,
        }
    }

    pub fn update(&mut self, hw: &mut dyn HardwareInterface) -> HttpUpdateStatus {
        self.update_counter += 1;

        if hw.millis() - self.sim7600_power_cycle_start_timestamp < 3000 {
            hw.set_digital_output(DigitalOutput::Sim7600PowerInhibit, true);
        } else {
            hw.set_digital_output(DigitalOutput::Sim7600PowerInhibit, false);
        }

        if hw.millis() - self.sim7600_power_cycle_start_timestamp < 4000 {
            return HttpUpdateStatus::Processing;
        }

        let ms_since_last_request = hw.millis() - self.last_http_request_millis;

        match hw.http_get_update() {
            HttpUpdateStatus::NotStarted => {
                if ms_since_last_request > 10000 || ms_since_last_request < 0 {
                    info!("http_get_update() -> NotStarted; starting");
                    hw.http_get_start(&self.url);
                    self.last_http_request_millis = hw.millis();
                }
                HttpUpdateStatus::NotStarted
            }
            HttpUpdateStatus::Processing => {
                // TODO: Update URL into hw here so that we're not storing
                //       old values for tens of seconds in the URL for sending
                //       later
                if self.update_counter % 100 == 0 {
                    info!("http_get_update() -> Processing");
                }
                HttpUpdateStatus::Processing
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
                HttpUpdateStatus::Failed(reason)
            }
            HttpUpdateStatus::Finished(response) => {
                info!("http_get_update() -> Finished; response: {:?}", response);
                hw.http_get_stop();
                HttpUpdateStatus::Finished(response)
            }
        }
    }
}
