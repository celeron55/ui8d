// Local modules
mod cli;
use cli::Cli;
mod sim7600simulator;
use sim7600simulator::Sim7600Simulator;

// Internal crates
use common::*;
use app::can_simulator::CanSimulator;

// Platform-specific dependencies
use ::image as im;
use ::image::ImageBuffer;
use ::image::Pixel;
use ::image::Rgba;
use clap::Parser;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use piston_window::*;

// Embedded-compatible libraries
use embedded_graphics as eg;
use embedded_graphics::{
    draw_target::DrawTarget, mono_font, pixelcolor::*, prelude::Point, text::Text, Drawable,
};
#[allow(unused_imports)]
use log::{info, warn};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

// General purpose libraries
use std::f64::consts::PI;
//use nalgebra::{Vector2, Point2, UnitComplex, Rotation2};
use arrayvec::ArrayString;

const FPS: u64 = 50;
const UPS: u64 = 50;

const DISPLAY_W: u32 = 320;
const DISPLAY_H: u32 = 240;
const DISPLAY_BORDER: u32 = 10;
const DISPLAY_SCALE: u32 = 1;

struct HardwareImplementation {
    ms_counter: u64,
    display: SimulatorDisplay<Rgb565>,
    sim7600sim: Sim7600Simulator,
    sim7600driver: Sim7600Driver,
    can_sim: CanSimulator,
}

impl HardwareImplementation {
    fn new() -> Self {
        Self {
            ms_counter: 0,
            display: SimulatorDisplay::<Rgb565>::new(eg::prelude::Size::new(DISPLAY_W, DISPLAY_H)),
            sim7600sim: Sim7600Simulator::new(),
            sim7600driver: Sim7600Driver::new(),
            can_sim: CanSimulator::new(),
        }
    }
}

impl HardwareImplementation {
    fn update_sim7600(&mut self) {
        self.sim7600driver.update_time(self.ms_counter);

        while let Some(b) = self.sim7600driver.buffers.txbuf.dequeue() {
            self.sim7600sim.push(b);
        }

        self.sim7600sim.update(self.ms_counter);

        while let Some(b) = self.sim7600sim.dequeue() {
            self.sim7600driver.push(b);
        }
    }
}

impl HardwareInterface for HardwareImplementation {
    fn millis(&mut self) -> u64 {
        self.ms_counter
    }

    fn display_clear(&mut self, color: Rgb565) {
        self.display.clear(color).unwrap();
    }

    fn display_draw_text(
        &mut self,
        text: &str,
        p: Point,
        style: mono_font::MonoTextStyle<Rgb565>,
        alignment: eg::text::Alignment,
    ) {
        Text::with_alignment(text, p, style, alignment)
            .draw(&mut self.display)
            .unwrap();
    }

    fn activate_dfu(&mut self) {
        warn!("activate_dfu() does nothing in desktop mode");
    }

    fn http_get_start(&mut self, url: &str) {
        info!("http_get_start(): url: {:?}", url);

        self.sim7600driver.http_get_start(url);
    }

    fn http_get_update(&mut self) -> HttpUpdateStatus {
        self.sim7600driver.http_get_update()
    }

    fn http_get_stop(&mut self) {
        self.sim7600driver.http_get_stop()
    }

    fn send_can(&mut self, frame: bxcan::Frame) {
        info!("send_can(): {:?}", frame);
    }

    fn get_analog_input(&mut self, input: AnalogInput) -> f32 {
        // TODO: ???
        14.0
    }

    fn set_digital_output(&mut self, output: DigitalOutput, value: bool) {
        info!("set_digital_output(): {:?}: {:?}", output, value);
        // TODO: Show somewhere
    }
}

fn main() {
    let cli = Cli::parse();

    stderrlog::new()
        .verbosity(log::LevelFilter::Info)
        .show_module_names(true)
        .module(module_path!())
        .module("common")
        .init()
        .unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let mut window: PistonWindow = WindowSettings::new(
        "ui8drust",
        [
            (DISPLAY_W + DISPLAY_BORDER * 2) * DISPLAY_SCALE,
            (DISPLAY_H + DISPLAY_BORDER * 2) * DISPLAY_SCALE,
        ],
    )
    .exit_on_esc(true)
    .build()
    .unwrap();

    let event_settings = EventSettings::new().max_fps(FPS).ups(UPS);
    let mut events = Events::new(event_settings);
    window.events = events;

    let mut state = app::MainState::new();
    state.store_log_for_display("See stderr for desktop log");

    let mut hw = HardwareImplementation::new();

    let mut counter: u64 = 0;

    while let Some(e) = window.next() {
        if e.render_args().is_some() {
            let output_settings = OutputSettingsBuilder::new().scale(DISPLAY_SCALE).build();
            let output_image = hw.display.to_rgb_output_image(&output_settings);
            let output_buffer = output_image.as_image_buffer();
            // Convert from Rgb to Rgba and from &[u8] to Vec<u8>
            let mut display_image: ImageBuffer<Rgba<u8>, Vec<u8>> =
                ImageBuffer::new(DISPLAY_W * DISPLAY_SCALE, DISPLAY_H * DISPLAY_SCALE);
            for y in 0..DISPLAY_H * DISPLAY_SCALE {
                for x in 0..DISPLAY_W * DISPLAY_SCALE {
                    let rgba = output_buffer.get_pixel(x, y).to_rgba();
                    display_image.put_pixel(x, y, rgba);
                }
            }
            let display_texture = Texture::from_image(
                &mut window.create_texture_context(),
                &display_image,
                &TextureSettings::new(),
            )
            .unwrap();

            window.draw_2d(&e, |c, g, _| {
                clear([0.0; 4], g);
                image(
                    &display_texture,
                    c.transform
                        .trans(DISPLAY_BORDER as f64, DISPLAY_BORDER as f64),
                    g,
                );
            });
        }

        if e.update_args().is_some() {
            hw.update_sim7600();

            hw.can_sim.update(hw.ms_counter);
            while let Some(frame) = hw.can_sim.txbuf.dequeue() {
                state.on_can(frame);
            }

            state.update(&mut hw);

            counter += 1;
            hw.ms_counter += 1000 / UPS;
        }

        if let Some(piston_window::Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::D1 | Key::Z => {
                    state.on_button_event(
                        common::ButtonEvent::ButtonPress(common::Button::Button1),
                        &mut hw,
                    );
                }
                Key::D2 | Key::X => {
                    state.on_button_event(
                        common::ButtonEvent::ButtonPress(common::Button::Button2),
                        &mut hw,
                    );
                }
                Key::D3 | Key::C => {
                    state.on_button_event(
                        common::ButtonEvent::ButtonPress(common::Button::Button3),
                        &mut hw,
                    );
                }
                Key::D4 | Key::V => {
                    state.on_button_event(
                        common::ButtonEvent::ButtonPress(common::Button::Button4),
                        &mut hw,
                    );
                }
                Key::D5 | Key::B => {
                    state.on_button_event(
                        common::ButtonEvent::ButtonPress(common::Button::Button5),
                        &mut hw,
                    );
                }
                Key::Q => {
                    break;
                }
                _ => {}
            }
        }
    }
}
