use log::{trace, warn};
use rand::random;

use crate::bus::KEYPAD_SIZE;

const V_SIZE: usize = 16;
const STACK_SIZE: usize = 16;
pub const SPRITE_ADDR: u16 = 0x000;
const PC_INIT: u16 = 0x0200;

pub struct Cpu {
    pc: u16,
    i: u16,
    v: [u8; V_SIZE], // v0..vf registers
    stack: Vec<u16>,
    key_await: Option<u8>,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            pc: PC_INIT,
            i: 0,
            v: [0; V_SIZE],
            stack: Vec::with_capacity(STACK_SIZE),
            key_await: None,
        }
    }

    fn pc_read_byte(&mut self, bus: &impl CpuBus) -> u8 {
        let byte = bus.read_byte(self.pc);
        self.pc = (self.pc + 1) & 0x0FFF;

        byte
    }

    fn pc_read_word(&mut self, bus: &impl CpuBus) -> u16 {
        let low = self.pc_read_byte(bus);
        let high = self.pc_read_byte(bus);

        (low as u16) << 8 | high as u16
    }

    pub fn emulate(&mut self, bus: &mut impl CpuBus) {
        if self.key_await.is_some() {
            let x = self.key_await.unwrap() as usize;

            for key in 0..KEYPAD_SIZE as u8 {
                if bus.read_keypad(key) {
                    self.v[x] = key;
                    self.key_await = None;
                    break;
                }
            }

            return;
        }

        let opcode = self.pc_read_word(bus);

        self.execute(bus, opcode);
    }

    pub fn reset(&mut self) {
        self.pc = PC_INIT;
        self.i = 0;
        for x in 0..V_SIZE {
            self.v[x] = 0;
        }
        self.stack.clear();
        self.key_await = None;
    }

    fn execute(&mut self, bus: &mut impl CpuBus, opcode: u16) {
        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );
        let nnn = (opcode & 0x0FFF) as u16;
        let nn = (opcode & 0x00FF) as u8;

        trace!("${:04x} : {:04x}", self.pc - 2, opcode);

        match nibbles {
            (0x0, 0x0, 0xe, 0x0) => self.opcode_00e0(bus),
            (0x0, 0x0, 0xe, 0xe) => self.opcode_00ee(),
            (0x0, _, _, _) => self.opcode_0nnn(nnn),
            (0x1, _, _, _) => self.opcode_1nnn(nnn),
            (0x2, _, _, _) => self.opcode_2nnn(nnn),
            (0x3, x, _, _) => self.opcode_3xnn(x, nn),
            (0x4, x, _, _) => self.opcode_4xnn(x, nn),
            (0x5, x, y, 0) => self.opcode_5xy0(x, y),
            (0x6, x, _, _) => self.opcode_6xnn(x, nn),
            (0x7, x, _, _) => self.opcode_7xnn(x, nn),
            (0x8, x, y, 0x0) => self.opcode_8xy0(x, y),
            (0x8, x, y, 0x1) => self.opcode_8xy1(x, y),
            (0x8, x, y, 0x2) => self.opcode_8xy2(x, y),
            (0x8, x, y, 0x3) => self.opcode_8xy3(x, y),
            (0x8, x, y, 0x4) => self.opcode_8xy4(x, y),
            (0x8, x, y, 0x5) => self.opcode_8xy5(x, y),
            (0x8, x, y, 0x6) => self.opcode_8xy6(x, y),
            (0x8, x, y, 0x7) => self.opcode_8xy7(x, y),
            (0x8, x, y, 0xe) => self.opcode_8xye(x, y),
            (0x9, x, y, 0x0) => self.opcode_9xy0(x, y),
            (0xa, _, _, _) => self.opcode_annn(nnn),
            (0xb, _, _, _) => self.opcode_bnnn(nnn),
            (0xc, x, _, _) => self.opcode_cxnn(x, nn),
            (0xd, x, y, n) => self.opcode_dxyn(x, y, n, bus),
            (0xe, x, 0x9, 0xe) => self.opcode_ex9e(x, bus),
            (0xe, x, 0xa, 0x1) => self.opcode_exa1(x, bus),
            (0xf, x, 0x0, 0x7) => self.opcode_fx07(x, bus),
            (0xf, x, 0x0, 0xa) => self.opcode_fx0a(x),
            (0xf, x, 0x1, 0x5) => self.opcode_fx15(x, bus),
            (0xf, x, 0x1, 0x8) => self.opcode_fx18(x, bus),
            (0xf, x, 0x1, 0xe) => self.opcode_fx1e(x),
            (0xf, x, 0x2, 0x9) => self.opcode_fx29(x),
            (0xf, x, 0x3, 0x3) => self.opcode_fx33(x, bus),
            (0xf, x, 0x5, 0x5) => self.opcode_fx55(x, bus),
            (0xf, x, 0x6, 0x5) => self.opcode_fx65(x, bus),
            _ => {}
        }
    }

    /// Execute machine language subroutine at address NNN
    fn opcode_0nnn(&mut self, nnn: u16) {
        trace!("not implemented, call {}", nnn);
    }

    /// Clear the screen
    fn opcode_00e0(&mut self, bus: &mut impl CpuBus) {
        bus.clear_screen();
    }

    /// Return from a subroutine
    fn opcode_00ee(&mut self) {
        match self.stack.pop() {
            Some(addr) => self.pc = addr,
            None => warn!("unable to return from subroutine"),
        }
    }

    /// Jump to address NNN
    fn opcode_1nnn(&mut self, nnn: u16) {
        self.pc = nnn & 0x0FFF;
    }

    /// Execute subroutine starting at address NNN
    fn opcode_2nnn(&mut self, nnn: u16) {
        self.stack.push(self.pc);
        self.pc = nnn;
    }

    /// Skip the following instruction if the value of register VX equals NN
    fn opcode_3xnn(&mut self, x: u8, nn: u8) {
        if self.v[x as usize] == nn {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    /// Skip the following instruction if the value of register VX is not equal
    /// to NN
    fn opcode_4xnn(&mut self, x: u8, nn: u8) {
        if self.v[x as usize] != nn {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    /// Skip the following instruction if the value of register VX is equal to
    /// the value of register VY
    fn opcode_5xy0(&mut self, x: u8, y: u8) {
        if self.v[x as usize] == self.v[y as usize] {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    /// Store number NN in register VX
    fn opcode_6xnn(&mut self, x: u8, nn: u8) {
        self.v[x as usize] = nn;
    }

    /// Add the value NN to register VX
    fn opcode_7xnn(&mut self, x: u8, nn: u8) {
        let x = x as usize;
        self.v[x] = self.v[x].wrapping_add(nn);
    }

    /// Store the value of register VY in register VX
    fn opcode_8xy0(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[y as usize];
    }

    /// Set VX to VX OR VY
    fn opcode_8xy1(&mut self, x: u8, y: u8) {
        self.v[x as usize] |= self.v[y as usize];
    }

    /// Set VX to VX AND VY
    fn opcode_8xy2(&mut self, x: u8, y: u8) {
        self.v[x as usize] &= self.v[y as usize];
    }

    /// Set VX to VX XOR VY
    fn opcode_8xy3(&mut self, x: u8, y: u8) {
        self.v[x as usize] ^= self.v[y as usize];
    }

    /// Add the value of register VY to register VX
    /// Set VF to 01 if a carry occurs
    /// Set VF to 00 if a carry does not occur
    fn opcode_8xy4(&mut self, x: u8, y: u8) {
        let x = x as usize;
        let y = y as usize;

        let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = res;
        self.v[0xF] = if overflow { 0x01 } else { 0x00 };
    }

    /// Subtract the value of register VY from register VX
    /// Set VF to 00 if a borrow occurs
    /// Set VF to 01 if a borrow does not occur
    fn opcode_8xy5(&mut self, x: u8, y: u8) {
        let x = x as usize;
        let y = y as usize;

        let (res, overflow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = res;
        self.v[0xF] = if overflow { 0x00 } else { 0x01 };
    }

    // Store the value of register VY shifted right one bit in register VX
    // Set register VF to the least significant bit prior to the shift
    // VY is unchanged
    fn opcode_8xy6(&mut self, x: u8, y: u8) {
        let x = x as usize;
        let y = y as usize;

        self.v[x] = self.v[y] >> 1;
        self.v[0xF] = self.v[y] & 0x01;
    }

    /// Set register VX to the value of VY minus VX
    /// Set VF to 00 if a borrow occurs
    /// Set VF to 01 if a borrow does not occur
    fn opcode_8xy7(&mut self, x: u8, y: u8) {
        let x = x as usize;
        let y = y as usize;

        let (res, overflow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = res;
        self.v[0xF] = if overflow { 0x00 } else { 0x01 };
    }

    /// Store the value of register VY shifted left one bit in register VX
    /// Set register VF to the most significant bit prior to the shift
    /// VY is unchanged
    fn opcode_8xye(&mut self, x: u8, y: u8) {
        let x = x as usize;
        let y = y as usize;

        self.v[x] = self.v[y] << 1;
        self.v[0xF] = (self.v[y] & 0x80) >> 7;
    }

    /// Skip the following instruction if the value of register VX is not
    /// equal to the value of register VY
    fn opcode_9xy0(&mut self, x: u8, y: u8) {
        if self.v[x as usize] != self.v[y as usize] {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    /// Store memory address NNN in register I
    fn opcode_annn(&mut self, nnn: u16) {
        self.i = nnn;
    }

    /// Jump to address NNN + V0
    fn opcode_bnnn(&mut self, nnn: u16) {
        self.pc = nnn.wrapping_add(self.v[0] as u16);
    }

    /// Set VX to a random number with a mask of NN
    fn opcode_cxnn(&mut self, x: u8, nn: u8) {
        self.v[x as usize] = random::<u8>() & nn;
    }

    /// Draw a sprite at position VX, VY with N bytes of sprite data starting
    /// at the address stored in I
    /// Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
    fn opcode_dxyn(&mut self, x: u8, y: u8, n: u8, bus: &mut impl CpuBus) {
        self.v[0xF] = 0x0;

        for h in 0..n as u8 {
            let sprite_line = bus.read_byte(self.i.wrapping_add(h as u16));
            let y = self.v[y as usize].wrapping_add(h);

            for w in 0..8_u8 {
                let x = self.v[x as usize].wrapping_add(w);

                let toggle = (sprite_line << w) & 0x80 > 0;

                if toggle {
                    let pixel = bus.read_screen(x, y);

                    if pixel {
                        self.v[0xF] = 0x1;
                    }

                    bus.write_screen(x, y, pixel ^ true);
                }
            }
        }
    }

    /// Skip the following instruction if the key corresponding to the hex
    /// value currently stored in register VX is pressed
    fn opcode_ex9e(&mut self, x: u8, bus: &impl CpuBus) {
        if bus.read_keypad(self.v[x as usize]) {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    /// Skip the following instruction if the key corresponding to the hex
    /// value currently stored in register VX is not pressed
    fn opcode_exa1(&mut self, x: u8, bus: &impl CpuBus) {
        if !bus.read_keypad(self.v[x as usize]) {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    /// Store the current value of the delay timer in register
    fn opcode_fx07(&mut self, x: u8, bus: &impl CpuBus) {
        self.v[x as usize] = bus.read_timer();
    }

    /// Wait for a keypress and store the result in register VX
    fn opcode_fx0a(&mut self, x: u8) {
        self.key_await = Some(x);
    }

    /// Set the delay timer to the value of register VX
    fn opcode_fx15(&mut self, x: u8, bus: &mut impl CpuBus) {
        bus.write_timer(self.v[x as usize]);
    }

    /// Set the sound timer to the value of register VX
    fn opcode_fx18(&mut self, x: u8, bus: &mut impl CpuBus) {
        bus.write_sound(self.v[x as usize]);
    }

    /// Add the value stored in register VX to register I
    fn opcode_fx1e(&mut self, x: u8) {
        self.i += self.v[x as usize] as u16;
        self.i &= 0x0FFF
    }

    /// Set I to the memory address of the sprite data corresponding to the
    /// hexadecimal digit stored in register VX
    fn opcode_fx29(&mut self, x: u8) {
        self.i = SPRITE_ADDR + self.v[x as usize] as u16 * 5;
        self.i &= 0x0FFF
    }

    /// Store the binary-coded decimal equivalent of the value stored in
    /// register VX at addresses I, I + 1, and I + 2
    fn opcode_fx33(&mut self, x: u8, bus: &mut impl CpuBus) {
        let value = self.v[x as usize];

        bus.write_byte(self.i + 2, value % 10);
        bus.write_byte(self.i + 1, (value / 10) % 10);
        bus.write_byte(self.i, value / 100);
    }

    /// Store the values of registers V0 to VX inclusive in memory starting
    /// at address I
    /// I is set to I + X + 1 after operation
    fn opcode_fx55(&mut self, x: u8, bus: &mut impl CpuBus) {
        for addr in 0..=x as u16 {
            bus.write_byte(self.i.wrapping_add(addr), self.v[addr as usize]);
        }

        self.i += x as u16 + 1;
        self.i &= 0x0FFF;
    }

    /// Fill registers V0 to VX inclusive with the values stored in memory
    /// starting at address I
    /// I is set to I + X + 1 after operation
    fn opcode_fx65(&mut self, x: u8, bus: &mut impl CpuBus) {
        for addr in 0..=x as u16 {
            self.v[addr as usize] = bus.read_byte(self.i.wrapping_add(addr));
        }

        self.i += x as u16 + 1;
        self.i &= 0x0FFF;
    }
}

pub trait CpuBus {
    // memory
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, byte: u8);

    // keypad
    fn read_keypad(&self, key: u8) -> bool;

    // screen
    fn clear_screen(&mut self);
    fn read_screen(&self, x: u8, y: u8) -> bool;
    fn write_screen(&mut self, x: u8, y: u8, pixel: bool);

    // timer
    fn read_timer(&self) -> u8;
    fn write_timer(&mut self, value: u8);

    // sound
    fn write_sound(&mut self, value: u8);
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const SCREEN_W: usize = 64;
    pub const SCREEN_H: usize = 32;

    struct BusTest {
        screen: Vec<Vec<bool>>,
        memory: Vec<u8>,
        keypad: Vec<bool>,
        timer: u8,
        sound: u8,
        clear_screen_call: usize,
    }

    impl CpuBus for BusTest {
        fn read_byte(&self, addr: u16) -> u8 {
            self.memory[addr as usize]
        }

        fn write_byte(&mut self, addr: u16, byte: u8) {
            self.memory[addr as usize] = byte;
        }

        fn read_keypad(&self, key: u8) -> bool {
            self.keypad[key as usize]
        }

        fn clear_screen(&mut self) {
            self.clear_screen_call += 1;
        }

        fn read_screen(&self, x: u8, y: u8) -> bool {
            self.screen[x as usize % SCREEN_W][y as usize % SCREEN_H]
        }

        fn write_screen(&mut self, x: u8, y: u8, pixel: bool) {
            self.screen[x as usize % SCREEN_W][y as usize % SCREEN_H] = pixel
        }

        fn read_timer(&self) -> u8 {
            self.timer
        }

        fn write_timer(&mut self, value: u8) {
            self.timer = value;
        }

        fn write_sound(&mut self, value: u8) {
            self.sound = value;
        }
    }

    fn create_cpu() -> Cpu {
        let mut cpu = Cpu::new();

        for x in 0..=0xE {
            cpu.v[x] = (42 + (1 + x * 0x5F)) as u8;
        }

        cpu
    }

    fn create_bus() -> BusTest {
        BusTest {
            screen: vec![vec![false; SCREEN_H]; SCREEN_W],
            memory: vec![0; 0x1000],
            keypad: vec![false; 16],
            timer: 0,
            sound: 0,
            clear_screen_call: 0,
        }
    }

    fn create_cpu_with_bus() -> (Cpu, BusTest) {
        (create_cpu(), create_bus())
    }

    fn clear_screen(bus: &mut BusTest) {
        for w in 0..SCREEN_W {
            for h in 0..SCREEN_H {
                bus.screen[w][h] = false;
            }
        }
    }

    #[test]
    fn test_opcode_0nnn() {
        let mut cpu = create_cpu();
        cpu.opcode_0nnn(0x0000);
    }

    #[test]
    fn test_opcode_00e0() {
        let (mut cpu, mut bus) = create_cpu_with_bus();
        bus.clear_screen_call = 0;

        cpu.opcode_00e0(&mut bus);
        assert_eq!(bus.clear_screen_call, 1);
    }

    #[test]
    fn test_opcode_00ee() {
        let mut cpu = create_cpu();

        cpu.pc = 0x100;
        cpu.stack.push(0x0200);

        cpu.opcode_00ee();

        assert_eq!(0x0200, cpu.pc);

        cpu.pc = 0x100;
        cpu.opcode_00ee(); // stack empty
        assert_eq!(0x100, cpu.pc);
    }

    #[test]
    fn test_opcode_1nnn() {
        let mut cpu = create_cpu();
        cpu.pc = 0x100;

        for nnn in 0..0x0FFF_u16 {
            cpu.opcode_1nnn(nnn);
            assert_eq!(nnn, cpu.pc); // jump to nnn
        }
    }

    #[test]
    fn test_opcode_2nnn() {
        let mut cpu = create_cpu();

        cpu.pc = 0x100;

        cpu.opcode_2nnn(0x0200);
        assert_eq!(0x0100, cpu.stack[0]);
        assert_eq!(0x0200, cpu.pc);

        cpu.opcode_2nnn(0x0FFF);
        assert_eq!(0x0100, cpu.stack[0]);
        assert_eq!(0x0200, cpu.stack[1]);
        assert_eq!(0x0FFF, cpu.pc);

        cpu.opcode_2nnn(0x0555);
        assert_eq!(0x0100, cpu.stack[0]);
        assert_eq!(0x0200, cpu.stack[1]);
        assert_eq!(0x0FFF, cpu.stack[2]);
        assert_eq!(0x0555, cpu.pc);
    }

    #[test]
    fn test_opcode_3xnn() {
        let mut cpu = create_cpu();

        for x in 0..=0xE {
            for nn in 0..=0xFF {
                cpu.pc = 0x100;
                cpu.v[x as usize] = nn;

                cpu.opcode_3xnn(x, nn);
                assert_eq!(0x102, cpu.pc); // jump

                cpu.opcode_3xnn(x, nn.wrapping_add(0x55));
                assert_eq!(0x102, cpu.pc); // no jump

                cpu.opcode_3xnn(x, nn);
                assert_eq!(0x104, cpu.pc); // jump
            }
        }
    }

    #[test]
    fn test_opcode_4xnn() {
        let mut cpu = create_cpu();

        for x in 0..=0xE {
            for nn in 0..=0xFF {
                cpu.pc = 0x100;
                cpu.v[x as usize] = nn;

                cpu.opcode_4xnn(x, nn);
                assert_eq!(0x100, cpu.pc); // no jump

                cpu.opcode_4xnn(x, nn.wrapping_add(0x55));
                assert_eq!(0x102, cpu.pc); // jump

                cpu.opcode_4xnn(x, nn);
                assert_eq!(0x102, cpu.pc); // no jump
            }
        }
    }

    #[test]
    fn test_opcode_5xy0() {
        let mut cpu = create_cpu();
        cpu.pc = 0x100;

        let x = 0u8;
        let y = 1u8;
        cpu.v[x as usize] = 0xFF;
        cpu.v[y as usize] = 0x55;

        cpu.opcode_5xy0(x, y);
        assert_eq!(0x100, cpu.pc); // no jump

        cpu.v[x as usize] = 0x55;
        cpu.opcode_5xy0(x, y);
        assert_eq!(0x102, cpu.pc); // jump
    }

    #[test]
    fn test_opcode_6xnn() {
        let mut cpu = create_cpu();

        for x in 0..=0xE {
            for nn in 0..=0xFF {
                cpu.opcode_6xnn(x, nn);
                assert_eq!(cpu.v[x as usize], nn);
            }
        }
    }

    #[test]
    fn test_opcode_7xnn() {
        let mut cpu = create_cpu();
        cpu.v[0] = 0x55;
        cpu.v[0xF] = 0x00;

        cpu.opcode_7xnn(0, 0x05);
        assert_eq!(cpu.v[0], 0x5A);
        assert_eq!(cpu.v[0xF], 0x0);

        cpu.opcode_7xnn(0, 0xAF); // overflow but no carry flag
        assert_eq!(cpu.v[0], 0x09);
        assert_eq!(cpu.v[0xF], 0x0);
    }

    #[test]
    fn test_opcode_8xy0() {
        for x in 0..=0xE {
            for y in 0..=0xE {
                let mut cpu = create_cpu();
                cpu.opcode_8xy0(x, y);
                assert_eq!(cpu.v[x as usize], cpu.v[y as usize]);
            }
        }
    }

    #[test]
    fn test_opcode_8xy1() {
        for x in 0..=0xE {
            for y in 0..=0xE {
                let mut cpu = create_cpu();
                let expected = cpu.v[x as usize] | cpu.v[y as usize];
                cpu.opcode_8xy1(x, y);
                assert_eq!(cpu.v[x as usize], expected);
            }
        }
    }

    #[test]
    fn test_opcode_8xy2() {
        for x in 0..=0xE {
            for y in 0..=0xE {
                let mut cpu = create_cpu();
                let expected = cpu.v[x as usize] & cpu.v[y as usize];

                cpu.opcode_8xy2(x, y);
                assert_eq!(cpu.v[x as usize], expected);
            }
        }
    }

    #[test]
    fn test_opcode_8xy3() {
        for x in 0..=0xE {
            for y in 0..=0xE {
                let mut cpu = create_cpu();
                let expected = cpu.v[x as usize] ^ cpu.v[y as usize];

                cpu.opcode_8xy3(x, y);
                assert_eq!(cpu.v[x as usize], expected);
            }
        }
    }

    #[test]
    fn test_opcode_8xy4() {
        let mut cpu = create_cpu();
        cpu.v[0] = 0x55;
        cpu.v[0xF] = 0x00;

        cpu.v[1] = 0x05;
        cpu.opcode_8xy4(0, 1);
        assert_eq!(cpu.v[0], 0x5A);
        assert_eq!(cpu.v[1], 0x05);
        assert_eq!(cpu.v[0xF], 0x0);

        cpu.v[1] = 0xAF;
        cpu.opcode_8xy4(0, 1); // overflow -> carry flag
        assert_eq!(cpu.v[0], 0x09);
        assert_eq!(cpu.v[1], 0xAF);
        assert_eq!(cpu.v[0xF], 0x01);

        cpu.v[1] = 0x05;
        cpu.opcode_8xy4(0, 1); // clear flag
        assert_eq!(cpu.v[0], 0x0E);
        assert_eq!(cpu.v[1], 0x05);
        assert_eq!(cpu.v[0xF], 0x00);

        //test with Vf
        cpu.v[0xF] = 0xF5;
        cpu.v[1] = 0x10;
        cpu.opcode_8xy4(0xF, 1);
        assert_eq!(cpu.v[1], 0x10);
        assert_eq!(cpu.v[0xF], 0x01);

        cpu.v[0xF] = 0x50;
        cpu.v[1] = 0x10;
        cpu.opcode_8xy4(0xF, 1);
        assert_eq!(cpu.v[1], 0x10);
        assert_eq!(cpu.v[0xF], 0x00);
    }

    #[test]
    fn test_opcode_8xy5() {
        let mut cpu = create_cpu();
        cpu.v[0] = 0x55;
        cpu.v[0xF] = 0x00;

        cpu.v[1] = 0x05;
        cpu.opcode_8xy5(0, 1);
        assert_eq!(cpu.v[0], 0x50);
        assert_eq!(cpu.v[0xF], 0x01);

        cpu.v[1] = 0x5A;
        cpu.opcode_8xy5(0, 1); // overflow -> carry flag
        assert_eq!(cpu.v[0], 0xF6);
        assert_eq!(cpu.v[0xF], 0x00);

        cpu.v[1] = 0x01;
        cpu.opcode_8xy5(0, 1); // clear flag
        assert_eq!(cpu.v[0], 0xF5);
        assert_eq!(cpu.v[0xF], 0x01);

        // test with Vf
        cpu.v[0xF] = 33;
        cpu.v[0] = 0x01;
        cpu.opcode_8xy5(0xF, 0);
        assert_eq!(cpu.v[0xF], 0x01);
        assert_eq!(cpu.v[0], 0x01);

        cpu.v[0xF] = 33;
        cpu.v[0] = 0x55;
        cpu.opcode_8xy5(0xF, 0);
        assert_eq!(cpu.v[0xF], 0x00);
        assert_eq!(cpu.v[0], 0x55);
    }

    #[test]
    fn test_opcode_8xy6() {
        let mut cpu = create_cpu();
        cpu.v[0] = 0x00;
        cpu.v[1] = 0x05;
        cpu.v[0xF] = 0x00;

        cpu.opcode_8xy6(0, 1);
        assert_eq!(cpu.v[0], 0x02);
        assert_eq!(cpu.v[1], 0x05);
        assert_eq!(cpu.v[0xF], 0x01);

        cpu.v[1] = 0x02;
        cpu.opcode_8xy6(0, 1);
        assert_eq!(cpu.v[0], 0x01);
        assert_eq!(cpu.v[1], 0x02);
        assert_eq!(cpu.v[0xF], 0x00);

        //test with Vf
        cpu.v[0xf] = 0x55;
        cpu.v[1] = 0x05;
        cpu.opcode_8xy6(0xf, 1);
        assert_eq!(cpu.v[1], 0x05);
        assert_eq!(cpu.v[0xF], 0x01);

        cpu.v[0xf] = 0x55;
        cpu.v[1] = 0x02;
        cpu.opcode_8xy6(0xf, 1);
        assert_eq!(cpu.v[1], 0x02);
        assert_eq!(cpu.v[0xF], 0x00);
    }

    #[test]
    fn test_opcode_8xy7() {
        let mut cpu = create_cpu();
        cpu.v[0] = 0x01;
        cpu.v[1] = 0x05;
        cpu.v[0xF] = 0x00;

        cpu.opcode_8xy7(0, 1);
        assert_eq!(cpu.v[0], 0x04);
        assert_eq!(cpu.v[1], 0x05);
        assert_eq!(cpu.v[0xF], 0x01);

        cpu.opcode_8xy7(0, 1);
        assert_eq!(cpu.v[0], 0x01);
        assert_eq!(cpu.v[1], 0x05);
        assert_eq!(cpu.v[0xF], 0x01);

        cpu.v[0] = 0x10;
        cpu.opcode_8xy7(0, 1);
        assert_eq!(cpu.v[0], 0xF5);
        assert_eq!(cpu.v[1], 0x05);
        assert_eq!(cpu.v[0xF], 0x00);

        cpu.v[0] = 0x02;
        cpu.opcode_8xy7(0, 1);
        assert_eq!(cpu.v[0], 0x03);
        assert_eq!(cpu.v[1], 0x05);
        assert_eq!(cpu.v[0xF], 0x01);

        cpu.v[0] = 0x05;
        cpu.opcode_8xy7(0, 1);
        assert_eq!(cpu.v[0], 0x00);
        assert_eq!(cpu.v[1], 0x05);
        assert_eq!(cpu.v[0xF], 0x01);

        cpu.v[0] = 0x06;
        cpu.opcode_8xy7(0, 1);
        assert_eq!(cpu.v[0], 0xFF);
        assert_eq!(cpu.v[1], 0x05);
        assert_eq!(cpu.v[0xF], 0x00);

        // test with VF
        cpu.v[0xF] = 0x23;
        cpu.v[0x1] = 0x05;
        cpu.opcode_8xy7(0xF, 1);
        assert_eq!(cpu.v[0xF], 0x00);
        assert_eq!(cpu.v[1], 0x05);

        cpu.v[0xF] = 0x23;
        cpu.v[0x1] = 0x33;
        cpu.opcode_8xy7(0xF, 1);
        assert_eq!(cpu.v[0xF], 0x01);
        assert_eq!(cpu.v[1], 0x33);
    }

    #[test]
    fn test_opcode_8xye() {
        let mut cpu = create_cpu();
        cpu.v[0] = 0xFF;
        cpu.v[1] = 0x50;
        cpu.v[0xF] = 0x00;

        cpu.opcode_8xye(0, 1);
        assert_eq!(cpu.v[0], 0xA0);
        assert_eq!(cpu.v[1], 0x50);
        assert_eq!(cpu.v[0xF], 0x00);

        cpu.v[1] = 0xA0;
        cpu.opcode_8xye(0, 1);
        assert_eq!(cpu.v[0], 0x40);
        assert_eq!(cpu.v[1], 0xA0);
        assert_eq!(cpu.v[0xF], 0x01);

        cpu.v[1] = 0x01;
        cpu.opcode_8xye(0, 1);
        assert_eq!(cpu.v[0], 0x02);
        assert_eq!(cpu.v[1], 0x01);
        assert_eq!(cpu.v[0xF], 0x00);

        //test with Vf

        cpu.v[1] = 0xA0;
        cpu.opcode_8xye(0xF, 1);
        assert_eq!(cpu.v[1], 0xA0);
        assert_eq!(cpu.v[0xF], 0x01);

        cpu.v[1] = 0x01;
        cpu.opcode_8xye(0xF, 1);
        assert_eq!(cpu.v[1], 0x01);
        assert_eq!(cpu.v[0xF], 0x00);
    }

    #[test]
    fn test_opcode_9xy0() {
        let mut cpu = create_cpu();
        cpu.pc = 0x0200;
        cpu.v[0] = 0xFF;
        cpu.v[1] = 0x50;

        cpu.opcode_9xy0(0, 1);
        assert_eq!(cpu.pc, 0x0202);

        cpu.v[0] = 0x22;
        cpu.v[1] = 0x22;
        cpu.opcode_9xy0(0, 1);
        assert_eq!(cpu.pc, 0x0202);
    }

    #[test]
    fn test_opcode_annn() {
        let mut cpu = create_cpu();
        cpu.i = 0x0200;

        cpu.opcode_annn(0x0123);
        assert_eq!(cpu.i, 0x0123);
    }

    #[test]
    fn test_opcode_bnnn() {
        let mut cpu = create_cpu();
        cpu.pc = 0x0200;
        cpu.v[0] = 0x11;

        cpu.opcode_bnnn(0x0123);
        assert_eq!(cpu.pc, 0x0134);
    }

    #[test]
    fn test_opcode_cxnn() {
        let mut cpu = create_cpu();
        cpu.pc = 0x0200;
        cpu.v[0] = 0x00;

        for _i in 0..1000 {
            cpu.opcode_cxnn(0, 0xF0);
            assert!(cpu.v[0] & 0x0F == 0);

            cpu.opcode_cxnn(0, 0xAA);
            assert!(cpu.v[0] & 0x55 == 0);
        }
    }

    #[test]
    fn test_opcode_dxyn_draw() {
        let (mut cpu, mut bus) = create_cpu_with_bus();

        //  draw sprite
        bus.memory[0x500] = 0b1000_0001;
        bus.memory[0x501] = 0b0100_0001;
        bus.memory[0x502] = 0b0010_0001;
        bus.memory[0x503] = 0b0001_0001;
        bus.memory[0x504] = 0b0000_1001;
        bus.memory[0x505] = 0b0000_0101;
        bus.memory[0x506] = 0b0000_0011;
        bus.memory[0x507] = 0b0000_0001;

        for n in 0..=0xF {
            clear_screen(&mut bus);

            cpu.i = 0x500;
            cpu.v[0] = 0;
            cpu.v[1] = 10;
            cpu.v[0xF] = 0xFF;
            cpu.opcode_dxyn(0, 1, n as u8, &mut bus);

            assert_eq!(cpu.v[0xF], 0x00);

            for h in 0..8_usize {
                let line = bus.memory[0x500 + h];
                for w in 0..8_usize {
                    assert_eq!(
                        bus.read_screen(w as u8, (h as u8).wrapping_add(10)),
                        (line << w) & 0x80 > 0 && h < n
                    );
                }
            }
        }
    }

    #[test]
    fn test_opcode_dxyn_collision() {
        let (mut cpu, mut bus) = create_cpu_with_bus();

        clear_screen(&mut bus);
        bus.memory[0x500] = 0b1000_0000;
        cpu.i = 0x500;
        cpu.v[0] = 0;
        cpu.v[1] = 0;
        cpu.v[0xF] = 0xFF;

        cpu.opcode_dxyn(0, 1, 1, &mut bus);
        assert_eq!(cpu.v[0xF], 0x00);
        assert_eq!(bus.screen[0][0], true);

        cpu.opcode_dxyn(0, 1, 1, &mut bus);
        assert_eq!(cpu.v[0xF], 0x01);
        assert_eq!(bus.screen[0][0], false);
    }

    #[test]
    fn test_opcode_dxyn_wrap_h() {
        let (mut cpu, mut bus) = create_cpu_with_bus();

        clear_screen(&mut bus);
        bus.memory[0x500] = 0b0000_0011;
        cpu.i = 0x500;
        cpu.v[0] = SCREEN_W as u8 - 7;
        cpu.v[1] = 0;
        cpu.v[0xF] = 0x00;

        cpu.opcode_dxyn(0, 1, 1, &mut bus);
        assert_eq!(cpu.v[0xF], 0x00);
        assert_eq!(bus.screen[0][0], true);
        assert_eq!(bus.screen[SCREEN_W - 1][0], true);

        // clear one pixel
        bus.memory[0x500] = 0b0000_0001;
        cpu.opcode_dxyn(0, 1, 1, &mut bus);
        assert_eq!(cpu.v[0xF], 0x01);
        assert_eq!(bus.screen[0][0], false);
        assert_eq!(bus.screen[SCREEN_W - 1][0], true);
    }

    #[test]
    fn test_opcode_dxyn_wrap_v() {
        let (mut cpu, mut bus) = create_cpu_with_bus();

        clear_screen(&mut bus);
        bus.memory[0x500] = 0b1000_0000;
        bus.memory[0x501] = 0b1000_0000;
        bus.memory[0x502] = 0b1000_0000;
        cpu.i = 0x500;
        cpu.v[0] = 0;
        cpu.v[1] = SCREEN_H as u8 - 2;
        cpu.v[0xF] = 0x00;

        cpu.opcode_dxyn(0, 1, 3, &mut bus);
        assert_eq!(cpu.v[0xF], 0x00);
        assert_eq!(bus.screen[0][0], true);
        assert_eq!(bus.screen[0][SCREEN_H - 1], true);
        assert_eq!(bus.screen[0][SCREEN_H - 2], true);

        // clear one pixel
        bus.memory[0x500] = 0b0000_0000;
        bus.memory[0x501] = 0b0000_0000;
        bus.memory[0x502] = 0b1000_0000;
        cpu.opcode_dxyn(0, 1, 3, &mut bus);
        assert_eq!(cpu.v[0xF], 0x01);
        assert_eq!(bus.screen[0][0], false);
        assert_eq!(bus.screen[0][SCREEN_H - 1], true);
        assert_eq!(bus.screen[0][SCREEN_H - 2], true);
    }

    #[test]
    fn test_opcode_ex9e() {
        let (mut cpu, mut bus) = create_cpu_with_bus();
        cpu.v[1] = 0x5;
        cpu.pc = 0x0300;
        bus.keypad[0x5] = false;

        cpu.opcode_ex9e(1, &mut bus);
        assert_eq!(cpu.pc, 0x0300);

        bus.keypad[0x5] = true;
        cpu.opcode_ex9e(1, &mut bus);
        assert_eq!(cpu.pc, 0x0302);
    }

    #[test]
    fn test_opcode_exa1() {
        let (mut cpu, mut bus) = create_cpu_with_bus();
        cpu.v[1] = 0x5;
        cpu.pc = 0x0300;
        bus.keypad[0x5] = false;

        cpu.opcode_exa1(1, &mut bus);
        assert_eq!(cpu.pc, 0x0302);

        bus.keypad[0x5] = true;
        cpu.opcode_exa1(1, &mut bus);
        assert_eq!(cpu.pc, 0x0302);
    }

    #[test]
    fn test_opcode_fx07() {
        let (mut cpu, mut bus) = create_cpu_with_bus();
        cpu.v[1] = 0x5;
        bus.timer = 0xA0;

        cpu.opcode_fx07(1, &mut bus);
        assert_eq!(cpu.v[1], 0xA0);
    }

    #[test]
    fn test_opcode_fx0a() {
        let (mut cpu, mut bus) = create_cpu_with_bus();

        for key in 0..=0xF {
            for x in 0..=0xF {
                cpu.pc = 0x400;
                cpu.v[x] = 0x00;
                cpu.opcode_fx0a(x as u8);

                for _clk in 0..1000 {
                    cpu.emulate(&mut bus);
                }
                assert_eq!(cpu.pc, 0x0400);

                bus.keypad[key] = true;
                cpu.emulate(&mut bus);

                assert_eq!(cpu.pc, 0x0400);
                assert_eq!(cpu.v[x], key as u8);

                cpu.emulate(&mut bus); // next intruction
                assert_eq!(cpu.pc, 0x0402);

                bus.keypad[key] = false;
            }
        }
    }

    #[test]
    fn test_opcode_fx15() {
        let (mut cpu, mut bus) = create_cpu_with_bus();
        cpu.v[1] = 0x55;
        cpu.opcode_fx15(1, &mut bus);
        assert_eq!(bus.timer, 0x55);
    }

    #[test]
    fn test_opcode_fx18() {
        let (mut cpu, mut bus) = create_cpu_with_bus();
        cpu.v[1] = 0x55;
        cpu.opcode_fx18(1, &mut bus);
        assert_eq!(bus.sound, 0x55);
    }

    #[test]
    fn test_opcode_fx1e() {
        let mut cpu = create_cpu();
        cpu.i = 0x0100;
        cpu.v[1] = 0x55;
        cpu.v[0xF] = 0x12;
        cpu.opcode_fx1e(1);

        assert_eq!(cpu.i, 0x155);
        assert_eq!(cpu.v[1], 0x55);
        assert_eq!(cpu.v[0xF], 0x12);

        cpu.i = 0x0FFF;
        cpu.v[1] = 0x01;
        cpu.opcode_fx1e(1);
        assert_eq!(cpu.i, 0x000);
        assert_eq!(cpu.v[1], 0x01);
        assert_eq!(cpu.v[0xF], 0x12);
    }

    #[test]
    fn test_opcode_fx29() {
        let mut cpu = create_cpu();
        cpu.i = 0x0100;

        cpu.v[1] = 0x0;
        cpu.opcode_fx29(1);
        assert_eq!(cpu.i, SPRITE_ADDR);

        cpu.v[1] = 0x1;
        cpu.opcode_fx29(1);
        assert_eq!(cpu.i, SPRITE_ADDR + 5);

        cpu.v[1] = 0x32;
        cpu.opcode_fx29(1);
        assert_eq!(cpu.i, SPRITE_ADDR + (0x32 * 5));

        cpu.v[1] = 0xFF;
        cpu.opcode_fx29(1);
        assert_eq!(cpu.i, SPRITE_ADDR + (0xFF * 5));
    }

    #[test]
    fn test_opcode_fx33() {
        let (mut cpu, mut bus) = create_cpu_with_bus();

        cpu.v[1] = 123;
        cpu.i = 0x0500;
        cpu.opcode_fx33(1, &mut bus);
        assert_eq!(bus.memory[cpu.i as usize], 1);
        assert_eq!(bus.memory[cpu.i as usize + 1], 2);
        assert_eq!(bus.memory[cpu.i as usize + 2], 3);

        cpu.v[1] = 255;
        cpu.i = 0x0500;
        cpu.opcode_fx33(1, &mut bus);
        assert_eq!(bus.memory[cpu.i as usize], 2);
        assert_eq!(bus.memory[cpu.i as usize + 1], 5);
        assert_eq!(bus.memory[cpu.i as usize + 2], 5);

        cpu.v[1] = 0;
        cpu.i = 0x0500;
        cpu.opcode_fx33(1, &mut bus);
        assert_eq!(bus.memory[cpu.i as usize], 0);
        assert_eq!(bus.memory[cpu.i as usize + 1], 0);
        assert_eq!(bus.memory[cpu.i as usize + 2], 0);
    }

    #[test]
    fn test_opcode_fx55() {
        let (mut cpu, mut bus) = create_cpu_with_bus();

        for x in 0..=0xF {
            cpu.v[x] = x as u8;
            bus.memory[0x500 + x] = 0x00;
        }

        for x in 0..=0xF {
            cpu.i = 0x500;
            cpu.opcode_fx55(x, &mut bus);
            for x in 0..=x as usize {
                assert_eq!(bus.memory[0x500 + x], cpu.v[x]);
            }

            assert_eq!(cpu.i, 0x500 + x as u16 + 1);
        }
    }

    #[test]
    fn test_opcode_fx65() {
        let (mut cpu, mut bus) = create_cpu_with_bus();

        for x in 0..=0xF {
            bus.memory[0x500 + x] = x as u8;
        }

        for x in 0..=0xF {
            cpu.i = 0x500;

            for x in 0..=0xF {
                cpu.v[x] = 0x00;
            }

            cpu.opcode_fx65(x, &mut bus);
            for x in 0..=x as usize {
                assert_eq!(cpu.v[x], bus.memory[0x500 + x]);
            }

            assert_eq!(cpu.i, 0x500 + x as u16 + 1);
        }
    }
}
