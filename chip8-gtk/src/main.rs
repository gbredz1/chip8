use std::{env, time::Instant};

use chip8::{
    beep::Beeper,
    bus::{Bus, DISPLAY_HEIGHT, DISPLAY_WIDTH},
    cpu::Cpu,
    delay::Delay,
    rom::Rom,
};
use log::debug;

fn main() {
    dotenv::dotenv().ok();
    env_logger::builder().format_timestamp_nanos().init();

    let args: Vec<String> = env::args().collect();

    let rom_path_default = String::from("roms/slipperyslope.ch8");
    let rom_path = args.get(1).unwrap_or(&rom_path_default);

    debug!("start");

    let rom = Rom::new_from(rom_path).expect("Failed to read rom file");

    debug!("loaded: {}", rom);

    let cpu = Cpu::new();
    let delay = Delay::new();
    let beeper = Beeper::new();
    let bus = Bus::new(rom);

    let chip8 = Emulator {
        cpu,
        delay,
        beeper,
        bus,
        loop_time: Instant::now(),
        cpu_cycles: 0.0,
        video_frames: 0.0,
        delay_update: 0.0,
        beep_update: 0.0,
        running: true,
        display_scale: 8.0,
        gilrs: gilrs::Gilrs::new().expect("GilRs init"),
    };

    chip8.run();
}
struct Emulator {
    // chip8
    cpu: Cpu,
    delay: Delay,
    beeper: Beeper,
    bus: Bus,
    //
    loop_time: Instant,
    cpu_cycles: f64,
    video_frames: f64,
    delay_update: f64,
    beep_update: f64,
    running: bool,
    display_scale: f64,
    //
    gilrs: gilrs::Gilrs,
}

use gtk::prelude::*;
use gtk::{
    cairo,
    glib::{self, clone},
};
use std::cell::RefCell;
use std::rc::Rc;

const FOREGROUND_COLOR: (f64, f64, f64) =
    (69.0 / 255., 115. / 255., 13. / 255.);

impl Emulator {
    fn run(self) {
        let application = gtk::Application::new(
            Some("app.chip8-gtk"),
            Default::default(),
        );

        let self_mut = Rc::new(RefCell::new(self));
        application.connect_activate(
            clone!(@strong self_mut => move |application| {
                self_mut.borrow_mut().build_ui(&self_mut, application);
            }),
        );
        application.run();
    }

    fn build_ui(
        &mut self,
        self_mut: &Rc<RefCell<Self>>,
        application: &gtk::Application,
    ) {
        let window = gtk::ApplicationWindow::builder()
            .application(application)
            .title("Chip8 GTK")
            .window_position(gtk::WindowPosition::Center)
            .default_width(800)
            .default_height(600)
            .build();

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window.add(&vbox);

        let vbox2 = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        vbox.add(&vbox2);

        let reset_button = gtk::Button::builder().label("Reset").build();
        vbox2.add(&reset_button);

        reset_button.connect_clicked(clone!(@weak self_mut => move |_| {
            self_mut.borrow_mut().reset();
        }));

        let pause_button = gtk::Button::builder().label("Pause").build();
        vbox2.add(&pause_button);
        pause_button.connect_clicked(clone!(@weak self_mut => move |btn| {
            let mut self_mut = self_mut.borrow_mut();
            self_mut.pause();
            match self_mut.running {
                true => btn.set_label("Pause"),
                false => btn.set_label("Continue"),
            };
        }));

        let vbox2 = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        vbox.add(&vbox2);
        let drawing_area = gtk::DrawingArea::builder()
            .width_request((DISPLAY_WIDTH as f64 * self.display_scale) as i32)
            .height_request((DISPLAY_HEIGHT as f64 * self.display_scale) as i32)
            .build();
        vbox2.add(&drawing_area);
        drawing_area.connect_draw(clone!(@weak self_mut => @default-return Inhibit(false), move |_, cr| {
            let res = self_mut.borrow().display_draw(cr);
            Inhibit(match res {
                Ok(_) => false,
                Err(_) => true,
            })
        }));

        window.add_tick_callback(
            clone!(@weak self_mut => @default-return Continue(true),  move |_, _| {
                self_mut.borrow_mut().tick(&drawing_area.clone());
                Continue(true)
            }),
        );

        window.connect_key_press_event(clone!(@weak self_mut => @default-return Inhibit(false), move |_, event_key| {
            self_mut.borrow_mut().keyboard_inputs(event_key.hardware_keycode(), true);
            Inhibit(false)
        }));

        window.connect_key_release_event(clone!(@weak self_mut => @default-return Inhibit(false), move |_, event_key| {
            self_mut.borrow_mut().keyboard_inputs(event_key.hardware_keycode(), false);
            Inhibit(false)
        }));

        window.show_all();
        window.activate_focus();
    }

    fn reset(&mut self) {
        self.cpu.reset();
    }

    fn pause(&mut self) {
        self.running ^= true;
    }

    fn display_draw(&self, cr: &cairo::Context) -> Result<(), cairo::Error> {
        //background
        cr.set_source_rgb(
            FOREGROUND_COLOR.0,
            FOREGROUND_COLOR.1,
            FOREGROUND_COLOR.2,
        );
        cr.paint()?;

        let mut surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, 64, 32)?;
        {
            let mut data = surface.data().expect("data");

            for h in 0..DISPLAY_HEIGHT {
                for w in 0..DISPLAY_WIDTH {
                    if !self.bus.vram[w][h] {
                        continue;
                    }

                    let index = (DISPLAY_WIDTH * h + w) * 4;
                    *data.get_mut(index).expect("pixel") = 13; // B
                    *data.get_mut(index + 1).expect("pixel") = 209; // G
                    *data.get_mut(index + 2).expect("pixel") = 124; // R
                    *data.get_mut(index + 3).expect("pixel") = 0x99; // A
                }
            }
        }
        surface.flush();

        let pattern = cairo::SurfacePattern::create(&surface);
        pattern.set_filter(cairo::Filter::Fast);
        cr.scale(self.display_scale, self.display_scale);

        cr.set_source(&pattern)?;
        cr.paint()?;

        Ok(())
    }

    fn tick(&mut self, area: &gtk::DrawingArea) {
        // Examine new events
        while let Some(gilrs::Event {
            id: _,
            event,
            time: _,
        }) = self.gilrs.next_event()
        {
            match event {
                gilrs::EventType::ButtonPressed(button, _code) => {
                    self.gamepad_input(button, true)
                }

                gilrs::EventType::ButtonReleased(button, _code) => {
                    self.gamepad_input(button, false)
                }

                _ => {}
            }
        }

        let delta = self.loop_time.elapsed().as_secs_f64();

        self.cpu_cycles += delta / 0.002; // 500Hz
        while self.cpu_cycles >= 1.0 && self.running {
            self.cpu_cycles -= 1.0;
            self.cpu.emulate(&mut self.bus);
        }

        self.video_frames += delta / 0.02; // 50Hz
        while self.video_frames >= 1.0 {
            self.video_frames -= 1.0;
            area.queue_draw();
        }

        self.delay_update += delta / 0.0166666666667; // 60 Hz
        while self.delay_update >= 1.0 && self.running {
            self.delay_update -= 1.0;

            self.delay.update(&mut self.bus);
        }

        self.beep_update += delta / 0.0166666666667; // 60 Hz
        while self.beep_update >= 1.0 && self.running {
            self.beep_update -= 1.0;

            self.beeper.update(&mut self.bus);

            // self.update_audio();
        }

        self.loop_time = Instant::now();
    }

    fn keyboard_inputs(&mut self, key: u16, val: bool) {
        match key {
            10 => self.bus.keys[0x1] = val,
            11 => self.bus.keys[0x2] = val,
            12 => self.bus.keys[0x3] = val,
            13 => self.bus.keys[0xC] = val,
            24 => self.bus.keys[0x4] = val,
            25 => self.bus.keys[0x5] = val,
            26 => self.bus.keys[0x6] = val,
            27 => self.bus.keys[0xD] = val,
            38 => self.bus.keys[0x7] = val,
            39 => self.bus.keys[0x8] = val,
            40 => self.bus.keys[0x9] = val,
            41 => self.bus.keys[0xE] = val,
            52 => self.bus.keys[0xA] = val,
            53 => self.bus.keys[0x0] = val,
            54 => self.bus.keys[0xB] = val,
            55 => self.bus.keys[0xF] = val,
            _ => {}
        }
    }

    fn gamepad_input(&mut self, button: gilrs::Button, val: bool) {
        debug!("button: {:?}, {}", button, val);

        match button {
            gilrs::Button::DPadUp => self.bus.keys[0x5] = val,
            gilrs::Button::DPadDown => self.bus.keys[0x8] = val,
            gilrs::Button::DPadLeft => self.bus.keys[0x7] = val,
            gilrs::Button::DPadRight => self.bus.keys[0x9] = val,
            gilrs::Button::South => self.bus.keys[0x6] = val,
            _ => {}
        }
    }
}
