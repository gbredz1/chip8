use crate::bus::Bus;

pub struct Beeper {
    beep: bool,
}

impl Beeper {
    pub fn new() -> Self {
        Self { beep: false }
    }

    pub fn update(&mut self, bus: &mut impl BeeperBus) {
        let value = bus.read_sound();

        self.beep = value > 0;

        if self.beep {
            bus.write_sound(value - 1);
        }
    }

    pub fn is_beeping(&self) -> bool {
        self.beep
    }
}

pub trait BeeperBus {
    fn write_sound(&mut self, value: u8);
    fn read_sound(&self) -> u8;
}

impl BeeperBus for Bus {
    fn read_sound(&self) -> u8 {
        self.beep
    }

    fn write_sound(&mut self, value: u8) {
        self.beep = value;
    }
}
