use crate::bus::Bus;

pub struct Delay {}

impl Delay {
    pub fn new() -> Self {
        Self {}
    }
}

impl Delay {
    pub fn update(&mut self, bus: &mut impl DelayBus) {
        let value = bus.read_delay();

        if value > 0 {
            bus.write_delay(value - 1);
        }
    }
}

pub trait DelayBus {
    fn write_delay(&mut self, value: u8);
    fn read_delay(&self) -> u8;
}

impl DelayBus for Bus {
    fn write_delay(&mut self, value: u8) {
        self.delay = value;
    }

    fn read_delay(&self) -> u8 {
        self.delay
    }
}
