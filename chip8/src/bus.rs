use crate::{
    cpu::{CpuBus, SPRITE_ADDR},
    rom::Rom,
};

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const KEYPAD_SIZE: usize = 16;

pub struct Bus {
    memory: [u8; 0x1000],
    pub vram: [[bool; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
    pub keys: [bool; KEYPAD_SIZE],
    pub delay: u8,
    pub beep: u8,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        let mut memory = [0; 0x1000];

        Bus::load_font4x5(&mut memory);

        for addr in 0..rom.size() {
            memory[0x200 + addr] = rom.read(addr as u16);
        }

        let vram = [[false; DISPLAY_HEIGHT]; DISPLAY_WIDTH];
        let keys = [false; KEYPAD_SIZE];

        Self {
            memory,
            vram,
            keys,
            delay: 0,
            beep: 0,
        }
    }

    fn load_font4x5(memory: &mut [u8]) {
        for i in 0..FONT4X5.len() {
            memory[i + SPRITE_ADDR as usize] = FONT4X5[i];
        }
    }
}

const FONT4X5: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
impl CpuBus for Bus {
    fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        self.memory[addr as usize] = byte;
    }

    fn read_keypad(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    fn clear_screen(&mut self) {
        for w in 0..DISPLAY_WIDTH {
            for h in 0..DISPLAY_HEIGHT {
                self.vram[w][h] = false;
            }
        }
    }

    fn read_screen(&self, x: u8, y: u8) -> bool {
        self.vram[x as usize % DISPLAY_WIDTH][y as usize % DISPLAY_HEIGHT]
    }

    fn write_screen(&mut self, x: u8, y: u8, pixel: bool) {
        self.vram[x as usize % DISPLAY_WIDTH][y as usize % DISPLAY_HEIGHT] =
            pixel;
    }

    fn read_timer(&self) -> u8 {
        self.delay
    }

    fn write_timer(&mut self, value: u8) {
        self.delay = value;
    }

    fn write_sound(&mut self, value: u8) {
        self.beep = value;
    }
}
