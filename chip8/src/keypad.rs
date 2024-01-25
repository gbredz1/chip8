use std::fmt::Display;

#[repr(u8)]
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum Keypad {
    Key0 = 0x0,
    Key1 = 0x1,
    Key2 = 0x2,
    Key3 = 0x3,
    Key4 = 0x4,
    Key5 = 0x5,
    Key6 = 0x6,
    Key7 = 0x7,
    Key8 = 0x8,
    Key9 = 0x9,
    KeyA = 0xA,
    KeyB = 0xB,
    KeyC = 0xC,
    KeyD = 0xD,
    KeyE = 0xE,
    KeyF = 0xF,
}

impl Display for Keypad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Keypad {{ key=")?;
        write!(
            f,
            "{}",
            match self {
                Keypad::Key0 => "0",
                Keypad::Key1 => "1",
                Keypad::Key2 => "2",
                Keypad::Key3 => "3",
                Keypad::Key4 => "4",
                Keypad::Key5 => "5",
                Keypad::Key6 => "6",
                Keypad::Key7 => "7",
                Keypad::Key8 => "8",
                Keypad::Key9 => "9",
                Keypad::KeyA => "A",
                Keypad::KeyB => "B",
                Keypad::KeyC => "C",
                Keypad::KeyD => "D",
                Keypad::KeyE => "E",
                Keypad::KeyF => "F",
            }
        )?;
        write!(f, " }}")
    }
}
