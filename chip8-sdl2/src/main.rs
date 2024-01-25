mod sdl2_frontend;

use chip8::{beep::Beeper, bus::Bus, cpu::Cpu, delay::Delay, rom::Rom};
use log::debug;

use std::env;

use crate::sdl2_frontend::SDL2Frontend;

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
    let beep = Beeper::new();
    let bus = Bus::new(rom);

    SDL2Frontend::new(cpu, delay, beep, bus).run();
}
