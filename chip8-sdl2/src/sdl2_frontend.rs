use std::{
    collections::HashMap,
    thread::sleep,
    time::{Duration, Instant},
};

use chip8::{
    beep::Beeper,
    bus::{Bus, DISPLAY_HEIGHT, DISPLAY_WIDTH},
    cpu::Cpu,
    delay::Delay,
    keypad::Keypad,
};
use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpecDesired, AudioStatus},
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::Canvas,
    video::Window,
    EventPump,
};

const FOREGROUND: Color = Color::RGB(69, 115, 13);
const BACKGROUND: Color = Color::RGB(124, 209, 21);

pub struct SDL2Frontend {
    // chip8
    cpu: Cpu,
    delay: Delay,
    beeper: Beeper,
    bus: Bus,
    // sdl
    canvas: Canvas<Window>,
    audio_device: AudioDevice<SquareWave>,
    event_pump: EventPump,
    // loop
    running: bool,
}

impl SDL2Frontend {
    pub fn new(cpu: Cpu, delay: Delay, beeper: Beeper, bus: Bus) -> Self {
        let sdl = sdl2::init().expect("SDL2 Init");

        let canvas = SDL2Frontend::create_canvas(&sdl);
        let audio_device = SDL2Frontend::create_audio(&sdl);
        let event_pump = sdl.event_pump().expect("SDL2: EventPump");

        Self {
            // chip8
            cpu,
            delay,
            beeper,
            bus,
            // sdl
            canvas,
            audio_device,
            event_pump,
            // loop
            running: true,
        }
    }

    pub fn run(&mut self) {
        let mut loop_time = Instant::now();
        let mut cpu_cycles = 0.0;
        let mut video_frames = 0.0;
        let mut delay_update = 0.0;
        let mut beep_update = 0.0;
        let mut delta: f64;

        let mut key_map = HashMap::new();
        key_map.insert(Keycode::Num1, Keypad::Key1);
        key_map.insert(Keycode::Num2, Keypad::Key2);
        key_map.insert(Keycode::Num3, Keypad::Key3);
        key_map.insert(Keycode::Num4, Keypad::KeyC);
        key_map.insert(Keycode::A, Keypad::Key4);
        key_map.insert(Keycode::Z, Keypad::Key5);
        key_map.insert(Keycode::E, Keypad::Key6);
        key_map.insert(Keycode::R, Keypad::KeyD);
        key_map.insert(Keycode::Q, Keypad::Key7);
        key_map.insert(Keycode::S, Keypad::Key8);
        key_map.insert(Keycode::D, Keypad::Key9);
        key_map.insert(Keycode::F, Keypad::KeyE);
        key_map.insert(Keycode::W, Keypad::KeyA);
        key_map.insert(Keycode::X, Keypad::Key0);
        key_map.insert(Keycode::C, Keypad::KeyB);
        key_map.insert(Keycode::V, Keypad::KeyF);

        'running: loop {
            self.read_events(&key_map);

            if !self.running {
                break 'running;
            }

            delta = loop_time.elapsed().as_secs_f64();

            cpu_cycles += delta / 0.002; // 500Hz
            while cpu_cycles >= 1.0 {
                cpu_cycles -= 1.0;
                self.cpu.emulate(&mut self.bus);
            }

            video_frames += delta / 0.02; // 50Hz
            while video_frames >= 1.0 {
                video_frames -= 1.0;
                self.update_canvas();
            }

            delay_update += delta / 0.0166666666667; // 60 Hz
            while delay_update >= 1.0 {
                delay_update -= 1.0;

                self.delay.update(&mut self.bus);
            }

            beep_update += delta / 0.0166666666667; // 60 Hz
            while beep_update >= 1.0 {
                beep_update -= 1.0;

                self.beeper.update(&mut self.bus);

                self.update_audio();
            }

            loop_time = Instant::now();

            sleep(Duration::from_millis(10));
        }
    }

    fn read_events(&mut self, keymap: &HashMap<Keycode, Keypad>) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => self.running = false,

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) = keymap.get(&keycode) {
                        self.bus.keys[(*key as usize)] = true;
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(key) = keymap.get(&keycode) {
                        self.bus.keys[(*key as usize)] = false;
                    }
                }
                _ => {}
            }
        }
    }

    fn update_canvas(&mut self) {
        self.canvas.set_draw_color(BACKGROUND);
        self.canvas.clear();
        self.canvas.set_draw_color(FOREGROUND);

        for w in 0..DISPLAY_WIDTH {
            for h in 0..DISPLAY_HEIGHT {
                if !self.bus.vram[w][h] {
                    continue;
                }

                self.canvas
                    .fill_rect(Rect::new(w as i32, h as i32, 1, 1))
                    .expect("draw pixel")
            }
        }
        self.canvas.present();
    }

    fn update_audio(&mut self) {
        if self.beeper.is_beeping() {
            if self.audio_device.status() != AudioStatus::Playing {
                self.audio_device.resume();
            }
        } else if self.audio_device.status() == AudioStatus::Playing {
            self.audio_device.pause();
        }
    }

    fn create_canvas(sdl: &sdl2::Sdl) -> Canvas<Window> {
        let pixel_size = 8;
        let video_subsystem = sdl.video().expect("SDL2: video");
        let window = video_subsystem
            .window(
                "chip8",
                DISPLAY_WIDTH as u32 * pixel_size,
                DISPLAY_HEIGHT as u32 * pixel_size,
            )
            .position_centered()
            .opengl()
            .build()
            .expect("SDL2: window");
        let mut canvas = window
            .into_canvas()
            .accelerated()
            .build()
            .expect("SDL2: Canvas");
        canvas
            .set_scale(pixel_size as f32, pixel_size as f32)
            .expect("Canvas scale");
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
        canvas
    }

    fn create_audio(sdl: &sdl2::Sdl) -> AudioDevice<SquareWave> {
        let audio_subsystem = sdl.audio().expect("SDL2: sound");
        let desired_spec = AudioSpecDesired {
            freq: Some(44_100),
            channels: Some(1),
            samples: None,
        };
        let audio_device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                // Show obtained AudioSpec
                println!("{:?}", spec);

                // initialize the audio callback
                SquareWave {
                    phase_inc: 440.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.25,
                }
            })
            .expect("open playback");
        audio_device
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

// impl Serialize for Keypad {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let p = match *self {
//             Keypad::Key0 => "0",
//             Keypad::Key1 => "1",
//             Keypad::Key2 => "2",
//             Keypad::Key3 => "3",
//             Keypad::Key4 => "4",
//             Keypad::Key5 => "5",
//             Keypad::Key6 => "6",
//             Keypad::Key7 => "7",
//             Keypad::Key8 => "8",
//             Keypad::Key9 => "9",
//             Keypad::KeyA => "A",
//             Keypad::KeyB => "B",
//             Keypad::KeyC => "C",
//             Keypad::KeyD => "D",
//             Keypad::KeyE => "E",
//             Keypad::KeyF => "F",
//         };
//         serializer.serialize_str(p)
//     }
// }
