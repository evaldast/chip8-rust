extern crate rand;

use rand::prelude::{Rng, SeedableRng};

struct Memory {
    pub ram: [u8; 4096]
}

struct Registers {
    pub v: [u8; 16],
    pub i: u16,
    pub program_counter: u16,
}

struct Graphics {
    gfx: [bool; 64 * 32],
    redraw: bool,
}

struct Timers {
    pub delay_timer: u8,
    pub sound_timer: u8,
}

struct Stack {
    pub stack: [u16; 16],
    pub stack_pointer: u16,
}

struct Keypad {
    pub keys: [bool; 16]
}

struct Chip8 {
    pub memory: Memory,
    pub registers: Registers,
    pub graphics: Graphics,
    pub timers: Timers,
    pub stack: Stack,
    pub keypad: Keypad,
    pub rng: rand::prelude::ThreadRng,
}

impl Memory {
    fn load_font_set(&mut self) {
        let font_set: [u8; 80] =
            [
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
                0xF0, 0x80, 0xF0, 0x80, 0x80  // F
            ];

        self.ram[..80].copy_from_slice(&font_set);
    }

    fn load_rom(&mut self, rom: &[u8; 4096 - 512]) {
        self.ram[512..].copy_from_slice(rom);
    }

    fn current_pixel_is_on(&self, register_i: u16, axis_x: u8, axis_y: u8) -> bool {
        self.ram[(register_i + axis_y as u16) as usize] & (0x80 >> axis_x) == 1
    }
}

impl Registers {
    fn set_register_v_f_value(&mut self, value: u8) {
        self.v[0xF] = value;
    }

    fn get_register_v_f_value(&self) -> u8 { self.v[0xF] }
}

impl Graphics {
    fn current_pixel_is_on(&self, coord_x: u8, axis_x: u8, coord_y: u8, axis_y: u8) -> bool {
        self.gfx[(coord_x + axis_x + ((coord_y + axis_y) * 64)) as usize]
    }

    fn change_pixel_value(&mut self, coord_x: u8, axis_x: u8, coord_y: u8, axis_y: u8) {
        let mut pixel = self.gfx[(coord_x + axis_x + ((coord_y + axis_y) * 64)) as usize];

        pixel = !pixel;
    }
}

trait OpCode {
    fn extract_nibble_value(self, nibble_place: u8) -> u8;
    fn extract_arguments(&self) -> [u16; 3];
    fn get_argument_sum<T>(self, range: T) -> u16 where T: std::slice::SliceIndex<[u16], Output=[u16]>;
}

impl OpCode for u16 {
    fn extract_nibble_value(self, nibble_place: u8) -> u8 {
        match nibble_place {
            1 => ((self & 0xF000) >> 12) as u8,
            2 => ((self & 0x0F00) >> 8) as u8,
            3 => ((self & 0x00F0) >> 4) as u8,
            4 => (self & 0x000F) as u8,
            _ => 0
        }
    }

    fn extract_arguments(&self) -> [u16; 3] {
        [
            self & 0x0F00,
            self & 0x00F0,
            self & 0x000F,
        ]
    }

    fn get_argument_sum<T>(self, range: T) -> u16 where T: std::slice::SliceIndex<[u16], Output=[u16]> {
        self.extract_arguments()[range]
            .iter()
            .fold(0, |a, &b| a + b)
    }
}

impl Chip8 {
    pub fn initialize() -> Chip8 {
        let mut chip8 = Chip8 {
            registers: Registers { program_counter: 0x200, i: 0, v: [0; 16] },
            memory: Memory { ram: [0; 4096] },
            graphics: Graphics { gfx: [false; 2048], redraw: false },
            timers: Timers { sound_timer: 0, delay_timer: 0 },
            stack: Stack { stack: [0; 16], stack_pointer: 0 },
            keypad: Keypad { keys: [false; 16] },
            rng: rand::thread_rng(),
        };

        chip8.memory.load_font_set();

        chip8
    }

    pub fn emulate_cycle(&mut self) {
        let op_code: u16 = self.fetch_op_code();

        // Execute Opcode
        self.execute_op_code(op_code);

        // Update timers
    }

    fn fetch_op_code(&self) -> u16 {
        (self.memory.ram[self.registers.program_counter as usize]
            << 8
            | self.memory.ram[(self.registers.program_counter + 1) as usize])
            as u16
    }

    pub fn execute_op_code(&mut self, op_code: u16) {
        match op_code {
            0x00EE => self.return_from_subroutine(),
            0x1000...0x1FFF => self.jump_to_location(op_code),
            0x2000...0x2FFF => self.call_subroutine(op_code),
            0x3000...0x3FFF => self.skip_next_if_vx_eq_kk(op_code),
            0x4000...0x4FFF => self.skip_next_if_vx_neq_kk(op_code),
            0x5000...0x5FF0 => self.skip_next_if_vx_eq_vy(op_code),
            0x6000...0x6FFF => self.set_vx_to(op_code),
            0x7000...0x7FFF => self.add_kk_to_vx(op_code),
            0x8000...0x8FFF => {
                match op_code.extract_nibble_value(4) {
                    0 => self.set_vx_to_vy(op_code),
                    1 => self.set_vx_to_vx_and_vy_bitwise_or(op_code),
                    2 => self.set_vx_to_vx_and_vy_bitwise_and(op_code),
                    3 => self.set_vx_to_vx_and_vy_bitwise_xor(op_code),
                    4 => self.set_vx_to_vx_and_vy_sum(op_code),
                    5 => self.set_vx_to_vx_and_vy_difference(op_code),
                    6 => self.set_vx_to_vx_shift_right(op_code),
                    7 => self.set_vx_to_vy_and_vx_difference(op_code),
                    0xE => self.set_vx_to_vx_shift_left(op_code),
                    _ => return
                }
            }
            0x9000...0x9FFF => self.skip_next_if_vx_neq_vy(op_code),
            0xA000...0xAFFF => self.set_register_i_address(op_code),
            0xB000...0xBFFF => self.jump_to_location_plus_v_0(op_code),
            0xC000...0xCFFF => self.set_vx_to_random_and_kk(op_code),
            0xD000...0xDFFF => self.draw_sprite(op_code),
            0xE000...0xEFFF => {
                match op_code.extract_nibble_value(3) {
                    9 => self.skip_next_if_key_pressed(op_code),
                    0xA => self.skip_next_if_key_not_pressed(op_code),
                    _ => return
                }
            },
            0xF000...0xFFFF => {
                match op_code.extract_nibble_value(3) {
                    0 => {
                        match op_code.extract_nibble_value(4) {
                            7 => self.set_vx_to_delay_timer(op_code),
                            0xA => self.await_key_and_store_to_value_vx(op_code),                            
                            _ => return
                        }
                    },
                    1 => {
                        match op_code.extract_nibble_value(4) {
                            5 => self.set_delay_timer_to_vx(op_code),
                            8 => self.set_sound_timer_to_vx(op_code),
                            0xE => self.set_i_to_sum_of_i_and_vx(op_code),
                            _ => return
                        }
                    }
                    _ => return
                }
            }
            _ => return
        }
    }

    fn return_from_subroutine(&mut self) {
        self.stack.stack_pointer -= 1;

        self.registers.program_counter = self.stack.stack[self.stack.stack_pointer as usize];
    }

    fn jump_to_location(&mut self, op_code: u16) {
        self.registers.program_counter = op_code.get_argument_sum(..);
    }

    fn call_subroutine(&mut self, op_code: u16) {
        self.stack.stack[self.stack.stack_pointer as usize] = self.registers.program_counter;
        self.stack.stack_pointer += 1;

        self.registers.program_counter = op_code.get_argument_sum(..);
    }

    fn skip_next_if_vx_eq_kk(&mut self, op_code: u16) {
        if self.registers.v[op_code.extract_nibble_value(2) as usize] == op_code.get_argument_sum(1..) as u8 {
            self.registers.program_counter += 2;
        }
    }

    fn skip_next_if_vx_neq_kk(&mut self, op_code: u16) {
        if self.registers.v[op_code.extract_nibble_value(2) as usize] != op_code.get_argument_sum(1..) as u8 {
            self.registers.program_counter += 2;
        }
    }

    fn skip_next_if_vx_eq_vy(&mut self, op_code: u16) {
        if self.registers.v[op_code.extract_nibble_value(2) as usize] == self.registers.v[op_code.extract_nibble_value(3) as usize] {
            self.registers.program_counter += 2;
        }
    }

    fn set_vx_to(&mut self, op_code: u16) {
        self.registers.v[op_code.extract_nibble_value(2) as usize] = op_code.get_argument_sum(1..) as u8;
    }

    fn add_kk_to_vx(&mut self, op_code: u16) {
        self.registers.v[op_code.extract_nibble_value(2) as usize] += op_code.get_argument_sum(1..) as u8;
    }

    fn set_vx_to_vy(&mut self, op_code: u16) {
        self.registers.v[op_code.extract_nibble_value(2) as usize] = self.registers.v[op_code.extract_nibble_value(3) as usize];
    }

    fn set_vx_to_vx_and_vy_bitwise_or(&mut self, op_code: u16) {
        self.registers.v[op_code.extract_nibble_value(2) as usize] |= self.registers.v[op_code.extract_nibble_value(3) as usize];
    }

    fn set_vx_to_vx_and_vy_bitwise_and(&mut self, op_code: u16) {
        self.registers.v[op_code.extract_nibble_value(2) as usize] &= self.registers.v[op_code.extract_nibble_value(3) as usize];
    }

    fn set_vx_to_vx_and_vy_bitwise_xor(&mut self, op_code: u16) {
        self.registers.v[op_code.extract_nibble_value(2) as usize] ^= self.registers.v[op_code.extract_nibble_value(3) as usize];
    }

    fn set_vx_to_vx_and_vy_sum(&mut self, op_code: u16) {
        let sum = self.registers.v[op_code.extract_nibble_value(2) as usize]
            .overflowing_add(self.registers.v[op_code.extract_nibble_value(3) as usize]);

        match sum.1 {
            true => self.registers.set_register_v_f_value(1),
            false => self.registers.set_register_v_f_value(0)
        }

        self.registers.v[op_code.extract_nibble_value(2) as usize] = sum.0;
    }

    fn set_vx_to_vx_and_vy_difference(&mut self, op_code: u16) {
        let difference = self.registers.v[op_code.extract_nibble_value(2) as usize]
            .overflowing_sub(self.registers.v[op_code.extract_nibble_value(3) as usize]);

        match difference.1 {
            true => self.registers.set_register_v_f_value(0),
            false => self.registers.set_register_v_f_value(1)
        }

        self.registers.v[op_code.extract_nibble_value(2) as usize] = difference.0;
    }

    fn set_vx_to_vx_shift_right(&mut self, op_code: u16) {
        self.registers.set_register_v_f_value(self.registers.v[op_code.extract_nibble_value(2) as usize] & 1);
        self.registers.v[op_code.extract_nibble_value(2) as usize] >>= 1;
    }

    fn set_vx_to_vy_and_vx_difference(&mut self, op_code: u16) {
        let difference = self.registers.v[op_code.extract_nibble_value(3) as usize]
            .overflowing_sub(self.registers.v[op_code.extract_nibble_value(2) as usize]);

        match difference.1 {
            true => self.registers.set_register_v_f_value(0),
            false => self.registers.set_register_v_f_value(1)
        }

        self.registers.v[op_code.extract_nibble_value(2) as usize] = difference.0;
    }

    fn set_vx_to_vx_shift_left(&mut self, op_code: u16) {
        self.registers.set_register_v_f_value(self.registers.v[op_code.extract_nibble_value(2) as usize] >> 7);
        self.registers.v[op_code.extract_nibble_value(2) as usize] <<= 1;
    }

    fn skip_next_if_vx_neq_vy(&mut self, op_code: u16) {
        if self.registers.v[op_code.extract_nibble_value(2) as usize] != self.registers.v[op_code.extract_nibble_value(3) as usize] {
            self.registers.program_counter += 2;
        }
    }

    fn set_register_i_address(&mut self, op_code: u16) {
        self.registers.i = op_code.get_argument_sum(..);
        self.registers.program_counter += 2;
    }

    fn jump_to_location_plus_v_0(&mut self, op_code: u16) {
        self.registers.program_counter = op_code.get_argument_sum(..) + self.registers.v[0] as u16;
    }

    fn set_vx_to_random_and_kk(&mut self, op_code: u16) {
        let random_byte: u8 = self.rng.gen();

        self.registers.v[op_code.extract_nibble_value(2) as usize] = op_code.get_argument_sum(1..) as u8 & random_byte
    }

    fn draw_sprite(&mut self, op_code: u16) {
        let coord_x: u8 = self.registers.v[op_code.extract_nibble_value(2) as usize];
        let coord_y: u8 = self.registers.v[op_code.extract_nibble_value(3) as usize];
        let height: u8 = self.registers.v[op_code.extract_nibble_value(4) as usize];

        self.registers.set_register_v_f_value(0);

        for axis_y in 0..height {
            for axis_x in 0..8 {
                if self.memory.current_pixel_is_on(self.registers.i, axis_x, axis_y) {
                    continue;
                }
                if self.graphics.current_pixel_is_on(coord_x, axis_x, coord_y, axis_y) {
                    self.registers.set_register_v_f_value(1);
                }

                self.graphics.change_pixel_value(coord_x, axis_x, coord_y, axis_y);
            }
        }

        self.registers.program_counter += 2;
    }

    fn skip_next_if_key_pressed(&mut self, op_code: u16) {
        if self.keypad.keys[op_code.extract_nibble_value(2) as usize] {
            self.registers.program_counter += 4;
        } else {
            self.registers.program_counter += 2;
        }
    }

    fn skip_next_if_key_not_pressed(&mut self, op_code: u16) {
        if !self.keypad.keys[op_code.extract_nibble_value(2) as usize] {
            self.registers.program_counter += 4;
        } else {
            self.registers.program_counter += 2;
        }
    }

    fn set_vx_to_delay_timer(&mut self, op_code: u16) {
        self.registers.v[op_code.extract_nibble_value(2) as usize] = self.timers.delay_timer;
    }

    fn await_key_and_store_to_value_vx(&mut self, op_code: u16) {
        loop {
            for key in self.keypad.keys.iter() {
                if *key == true {
                    self.registers.v[op_code.extract_nibble_value(2) as usize] = 1;

                    return;
                }
            }
        }
    }

    fn set_delay_timer_to_vx(&mut self, op_code: u16) {
        self.timers.delay_timer = self.registers.v[op_code.extract_nibble_value(2) as usize];
    }

    fn set_sound_timer_to_vx(&mut self, op_code: u16) {
        self.timers.sound_timer = self.registers.v[op_code.extract_nibble_value(2) as usize];
    }

    fn set_i_to_sum_of_i_and_vx(& mut self, op_code: u16) {
        self.registers.i += self.registers.v[op_code.extract_nibble_value(2) as usize] as u16;
    }

    fn clear_screen(&mut self) {
        self.graphics.gfx = [false; 2048];
    }
}

fn main() {
    println!("Hello, chip8!");

    loop {}
}

#[cfg(test)]
mod tests {
    use super::{Chip8, OpCode};

    #[test]
    fn can_extract_nibble_value_correctly() {
        let test_value: u16 = 0x5FE2;

        assert_eq!(test_value.extract_nibble_value(1), 5);
        assert_eq!(test_value.extract_nibble_value(2), 0xF);
        assert_eq!(test_value.extract_nibble_value(3), 0xE);
        assert_eq!(test_value.extract_nibble_value(1), 5);
        assert_eq!(test_value.extract_nibble_value(15), 0);
    }

    #[test]
    fn can_extract_arguments_correctly() {
        let test_arguments: [u16; 3] = 0x12FE.extract_arguments();

        assert_eq!(test_arguments[0], 0x200)
    }

    //Return from a subroutine.
    //The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    #[test]
    fn can_process_op_00ee() {
        let mut chip8 = Chip8::initialize();
        let current_program_counter = 0x2DAB;
        let current_stack_pointer = 3;

        chip8.registers.program_counter = current_program_counter;
        chip8.stack.stack_pointer = current_stack_pointer;

        chip8.execute_op_code(0x2001);
        chip8.execute_op_code(0x00EE);

        assert_eq!(chip8.registers.program_counter, current_program_counter);
        assert_eq!(chip8.stack.stack_pointer, current_stack_pointer);
    }

    //Jump to location nnn.
    //The interpreter sets the program counter to nnn.
    #[test]
    fn can_process_op_1_nnn() {
        let mut chip8 = Chip8::initialize();
        let target_program_counter = 0x0FFE;

        chip8.execute_op_code(0x1FFE);

        assert_eq!(chip8.registers.program_counter, target_program_counter);
    }

    //Call subroutine at nnn.
    //The interpreter increments the stack pointer, then puts the current PC on the top of the stack.
    //The PC is then set to nnn.
    #[test]
    fn can_process_op_2_nnn() {
        let mut chip8 = Chip8::initialize();
        let current_stack_pointer = 5;
        let current_program_counter = 0x2EEE;

        chip8.stack.stack_pointer = current_stack_pointer;
        chip8.registers.program_counter = current_program_counter;

        chip8.execute_op_code(0x2111);

        assert_eq!(chip8.registers.program_counter, 0x111);
        assert_eq!(chip8.stack.stack_pointer, current_stack_pointer + 1);
        assert_eq!(chip8.stack.stack[current_stack_pointer as usize], current_program_counter);
    }

    //Skip next instruction if Vx = kk.
    //The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    #[test]
    fn can_process_op_3_xkk() {
        let mut chip8 = Chip8::initialize();
        let current_v_index = 2;
        let current_v_value = 0xAA;
        let current_program_counter = 0x0EE4;

        chip8.registers.v[current_v_index] = current_v_value;
        chip8.registers.program_counter = current_program_counter;

        chip8.execute_op_code(0x32AA);

        assert_eq!(chip8.registers.program_counter, 0x0EE6);

        chip8.execute_op_code(0x32AB);

        assert_eq!(chip8.registers.program_counter, 0x0EE6);
    }

    // Skip next instruction if Vx != kk.
    //The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    #[test]
    fn can_process_op_4_xkk() {
        let mut chip8 = Chip8::initialize();
        let current_v_index = 2;
        let current_v_value = 0xAA;
        let current_program_counter = 0x0EE4;

        chip8.registers.v[current_v_index] = current_v_value;
        chip8.registers.program_counter = current_program_counter;

        chip8.execute_op_code(0x42AA);

        assert_eq!(chip8.registers.program_counter, 0x0EE4);

        chip8.execute_op_code(0x42AB);

        assert_eq!(chip8.registers.program_counter, 0x0EE6);
    }

    //Skip next instruction if Vx = Vy.
    //The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    #[test]
    fn can_process_op_5_xy0() {
        let mut chip8 = Chip8::initialize();
        let current_v_index_first = 2;
        let current_v_value_first = 0xAA;
        let current_v_index_second = 5;
        let current_v_value_second = 0xAB;
        let current_program_counter = 0x0EE4;

        chip8.registers.v[current_v_index_first] = current_v_value_first;
        chip8.registers.v[current_v_index_second] = current_v_value_first;
        chip8.registers.program_counter = current_program_counter;

        chip8.execute_op_code(0x5250);

        assert_eq!(chip8.registers.program_counter, 0x0EE6);

        chip8.registers.v[current_v_index_first] = current_v_value_first;
        chip8.registers.v[current_v_index_second] = current_v_value_second;

        assert_eq!(chip8.registers.program_counter, 0x0EE6);
    }

    //Set Vx = kk.
    //The interpreter puts the value kk into register Vx.
    #[test]
    fn can_process_op_6_xkk() {
        let mut chip8 = Chip8::initialize();
        let current_v_index = 2;

        chip8.execute_op_code(0x62FF);

        assert_eq!(chip8.registers.v[current_v_index], 0x0FF);
    }

    //Set Vx = Vx + kk.
    //Adds the value kk to the value of register Vx, then stores the result in Vx.
    #[test]
    fn can_process_op_7_xkk() {
        let mut chip8 = Chip8::initialize();
        let current_v_index = 2;
        let current_v_value = 0xAA;

        chip8.registers.v[current_v_index] = current_v_value;

        chip8.execute_op_code(0x7211);

        assert_eq!(chip8.registers.v[current_v_index], 0xBB);
    }

    //Set Vx = Vy.
    //Stores the value of register Vy in register Vx.
    #[test]
    fn can_process_op_8_xy_0() {
        let mut chip8 = Chip8::initialize();
        let current_v_index_first = 2;
        let current_v_value_first = 0xAA;
        let current_v_index_second = 5;
        let current_v_value_second = 0xBB;

        chip8.registers.v[current_v_index_first] = current_v_value_first;
        chip8.registers.v[current_v_index_second] = current_v_value_second;

        chip8.execute_op_code(0x8250);

        assert_eq!(chip8.registers.v[current_v_index_first], current_v_value_second);
        assert_eq!(chip8.registers.v[current_v_index_second], current_v_value_second);
    }

    //Set Vx = Vx OR Vy.
    //Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
    #[test]
    fn can_process_op_8_xy_1() {
        let mut chip8 = Chip8::initialize();
        let current_v_index_first = 2;
        let current_v_value_first = 0xCC;
        let current_v_index_second = 5;
        let current_v_value_second = 0xBB;

        chip8.registers.v[current_v_index_first] = current_v_value_first;
        chip8.registers.v[current_v_index_second] = current_v_value_second;

        chip8.execute_op_code(0x8251);

        assert_eq!(chip8.registers.v[current_v_index_first], 0xFF);
        assert_eq!(chip8.registers.v[current_v_index_second], current_v_value_second);
    }

    //Set Vx = Vx AND Vy.
    //Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx
    #[test]
    fn can_process_op_8_xy_2() {
        let mut chip8 = Chip8::initialize();
        let current_v_index_first = 2;
        let current_v_value_first = 0x11;
        let current_v_index_second = 5;
        let current_v_value_second = 0x01;

        chip8.registers.v[current_v_index_first] = current_v_value_first;
        chip8.registers.v[current_v_index_second] = current_v_value_second;

        chip8.execute_op_code(0x8252);

        assert_eq!(chip8.registers.v[current_v_index_first], 0x1);
        assert_eq!(chip8.registers.v[current_v_index_second], current_v_value_second);
    }

    //Set Vx = Vx XOR Vy.
    //Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx
    #[test]
    fn can_process_op_8_xy_3() {
        let mut chip8 = Chip8::initialize();
        let current_v_index_first = 2;
        let current_v_value_first = 0x11;
        let current_v_index_second = 5;
        let current_v_value_second = 0x01;

        chip8.registers.v[current_v_index_first] = current_v_value_first;
        chip8.registers.v[current_v_index_second] = current_v_value_second;

        chip8.execute_op_code(0x8253);

        assert_eq!(chip8.registers.v[current_v_index_first], 0x10);
        assert_eq!(chip8.registers.v[current_v_index_second], current_v_value_second);
    }

    //Set Vx = Vx + Vy, set VF = carry.
    //The values of Vx and Vy are added together.
    //If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0.
    //Only the lowest 8 bits of the result are kept, and stored in Vx.
    #[test]
    fn can_process_op_8_xy_4() {
        let mut chip8 = Chip8::initialize();
        let current_v_index_first = 2;
        let current_v_value_first = 0xFF;
        let current_v_index_second = 5;
        let current_v_value_second = 0x03;

        chip8.registers.v[current_v_index_first] = current_v_value_first;
        chip8.registers.v[current_v_index_second] = current_v_value_second;

        chip8.execute_op_code(0x8254);

        assert_eq!(chip8.registers.v[current_v_index_first], 0x02);
        assert_eq!(chip8.registers.get_register_v_f_value(), 1);
        assert_eq!(chip8.registers.v[current_v_index_second], current_v_value_second);

        chip8.execute_op_code(0x8254);

        assert_eq!(chip8.registers.v[current_v_index_first], 0x05);
        assert_eq!(chip8.registers.get_register_v_f_value(), 0);
    }

    //Set Vx = Vx - Vy, set VF = NOT borrow.
    //If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    #[test]
    fn can_process_op_8_xy_5() {
        let mut chip8 = Chip8::initialize();
        let current_v_index_first = 2;
        let current_v_value_first = 0xAA;
        let current_v_index_second = 5;
        let current_v_value_second = 0xBB;

        chip8.registers.v[current_v_index_first] = current_v_value_first;
        chip8.registers.v[current_v_index_second] = current_v_value_second;

        chip8.execute_op_code(0x8255);

        assert_eq!(chip8.registers.v[current_v_index_first], 0xEF);
        assert_eq!(chip8.registers.get_register_v_f_value(), 0);
        assert_eq!(chip8.registers.v[current_v_index_second], current_v_value_second);

        chip8.execute_op_code(0x8255);

        assert_eq!(chip8.registers.v[current_v_index_first], 0x34);
        assert_eq!(chip8.registers.get_register_v_f_value(), 1);
    }

    //Set Vx = Vx SHR 1.
    //If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    #[test]
    fn can_process_op_8_xy_6() {
        let mut chip8 = Chip8::initialize();
        let current_v_index = 2;
        let current_v_value = 0xA1;

        chip8.registers.v[current_v_index] = current_v_value;
        chip8.execute_op_code(0x8256);

        assert_eq!(chip8.registers.v[current_v_index], 0x50);
        assert_eq!(chip8.registers.get_register_v_f_value(), 1);

        chip8.execute_op_code(0x8256);

        assert_eq!(chip8.registers.v[current_v_index], 0x28);
        assert_eq!(chip8.registers.get_register_v_f_value(), 0);
    }

    //Set Vx = Vy - Vx, set VF = NOT borrow.
    //If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
    #[test]
    fn can_process_op_8_xy_7() {
        let mut chip8 = Chip8::initialize();
        let current_v_index_first = 2;
        let current_v_value_first = 0xBB;
        let current_v_index_second = 5;
        let current_v_value_second = 0xAA;

        chip8.registers.v[current_v_index_first] = current_v_value_first;
        chip8.registers.v[current_v_index_second] = current_v_value_second;
        chip8.execute_op_code(0x8257);

        assert_eq!(chip8.registers.v[current_v_index_first], 0xEF);
        assert_eq!(chip8.registers.get_register_v_f_value(), 0);
        assert_eq!(chip8.registers.v[current_v_index_second], current_v_value_second);

        chip8.registers.v[current_v_index_first] = 0x11;

        chip8.execute_op_code(0x8257);

        assert_eq!(chip8.registers.v[current_v_index_first], 0x99);
        assert_eq!(chip8.registers.get_register_v_f_value(), 1);
    }

    //Set Vx = Vx SHL 1.
    //If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    #[test]
    fn can_process_op_8_xy_e() {
        let mut chip8 = Chip8::initialize();
        let current_v_index = 2;
        let current_v_value = 0x7F;

        chip8.registers.v[current_v_index] = current_v_value;
        chip8.execute_op_code(0x825E);

        assert_eq!(chip8.registers.v[current_v_index], 0xFE);
        assert_eq!(chip8.registers.get_register_v_f_value(), 0);

        chip8.execute_op_code(0x825E);

        assert_eq!(chip8.registers.v[current_v_index], 0xFC);
        assert_eq!(chip8.registers.get_register_v_f_value(), 1);
    }

    //Skip next instruction if Vx != Vy.
    //The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    #[test]
    fn can_process_op_9_xy_0() {
        let mut chip8 = Chip8::initialize();
        let current_v_index_first = 2;
        let current_v_value_first = 0xBB;
        let current_v_index_second = 5;
        let current_v_value_second = 0xAA;
        let current_program_counter = 0xA2;

        chip8.registers.v[current_v_index_first] = current_v_value_first;
        chip8.registers.v[current_v_index_second] = current_v_value_second;
        chip8.registers.program_counter = current_program_counter;

        chip8.execute_op_code(0x9250);

        assert_eq!(chip8.registers.program_counter, 0xA4);

        chip8.registers.v[current_v_index_second] = current_v_value_first;

        chip8.execute_op_code(0x9250);

        assert_eq!(chip8.registers.program_counter, 0xA4);
    }

    //Set I = nnn.
    //The value of register I is set to nnn.
    #[test]
    fn can_process_op_a_nnn() {
        let mut chip8 = Chip8::initialize();

        chip8.execute_op_code(0xA2F0);

        assert_eq!(chip8.registers.i, 0x02F0);
    }

    //Jump to location nnn + V0.
    //The program counter is set to nnn plus the value of V0.
    #[test]
    fn can_process_op_b_nnn() {
        let mut chip8 = Chip8::initialize();
        chip8.registers.v[0] = 0xAA;

        chip8.execute_op_code(0xB123);

        assert_eq!(chip8.registers.program_counter, 0x01CD);
    }

    //Set Vx = random byte AND kk.
    //The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
//    #[test]
//    fn can_process_op_c_x_kk() {
//        let mut chip8 = Chip8::initialize();
//        let current_v_index = 2;
//        let current_v_value = 0x11;
//
//        chip8.registers.v[current_v_index] = current_v_value;
//
//        chip8.execute_op_code(0xC201);
//
//        assert_eq!(chip8.registers.v[current_v_index], 0x10);
//    }

    //The interpreter reads n bytes from memory, starting at the address stored in I.
    //These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
    //Sprites are XORed onto the existing screen.
    //If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
    //If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen.
    #[test]
    fn can_process_op_d_xyn() {
        let mut chip8 = Chip8::initialize();

        chip8.execute_op_code(0xD003);

        //assert_eq!(chip8.memory.ram[chip8.registers.i as usize], 0x3C);
    }

    //Skip next instruction if key with the value of Vx is pressed.
    //Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
    #[test]
    fn can_process_op_e_x_9e() {
        let mut chip8 = Chip8::initialize();
        let current_program_counter = 0xA2;

        chip8.registers.program_counter = current_program_counter;
        chip8.keypad.keys[0x2] = true;
        chip8.keypad.keys[0xF] = true;

        chip8.execute_op_code(0xE29E);

        assert_eq!(chip8.registers.program_counter, 0xA6);

        chip8.execute_op_code(0xE19F);

        assert_eq!(chip8.registers.program_counter, 0xA8);

        chip8.execute_op_code(0xEF90);

        assert_eq!(chip8.registers.program_counter, 0xAC);
    }

    #[test]
    fn can_process_op_e_x_a1() {
        let mut chip8 = Chip8::initialize();
        let current_program_counter = 0xA2;

        chip8.registers.program_counter = current_program_counter;
        chip8.keypad.keys[0x2] = true;
        chip8.keypad.keys[0xF] = true;

        chip8.execute_op_code(0xE2AE);

        assert_eq!(chip8.registers.program_counter, 0xA4);

        chip8.execute_op_code(0xE1AF);

        assert_eq!(chip8.registers.program_counter, 0xA8);

        chip8.execute_op_code(0xEFA0);

        assert_eq!(chip8.registers.program_counter, 0xAA);
    }

    //Set Vx = delay timer value.
    //The value of DT is placed into Vx.
    #[test]
    fn can_process_op_f_x_07() {
        let mut chip8 = Chip8::initialize();
        let current_delay_timer_value = 0xFA;

        chip8.timers.delay_timer = current_delay_timer_value;

        chip8.execute_op_code(0xF507);

        assert_eq!(chip8.registers.v[5], current_delay_timer_value);
    }

    //Wait for a key press, store the value of the key in Vx.
    //All execution stops until a key is pressed, then the value of that key is stored in Vx.
    #[test]
    fn can_process_op_f_x_0a() {
        let mut chip8 = Chip8::initialize();

        chip8.keypad.keys[0x2] = true;

        assert_eq!(chip8.registers.v[5], 0);

        chip8.execute_op_code(0xF50A);

        assert_eq!(chip8.registers.v[5], 1);
    }

    //Set delay timer = Vx.
    //DT is set equal to the value of Vx.
    #[test]
    fn can_process_op_f_x_15() {
        let mut chip8 = Chip8::initialize();
        let current_v_index = 5;

        chip8.registers.v[current_v_index] = 0xAF; 
        chip8.timers.delay_timer = 0xFA;

        chip8.execute_op_code(0xF515);

        assert_eq!(chip8.registers.v[current_v_index], chip8.timers.delay_timer);
    }

    //Set sound timer = Vx.
    //ST is set equal to the value of Vx.
    #[test]
    fn can_process_op_f_x_18() {
        let mut chip8 = Chip8::initialize();
        let current_v_index = 3;
        let current_v_value = 0xCA;

        chip8.registers.v[current_v_index] = current_v_value;

        chip8.execute_op_code(0xF318);

        assert_eq!(chip8.timers.sound_timer, current_v_value);
    }

    // Set I = I + Vx.
    //The values of I and Vx are added, and the results are stored in I.
    #[test]
    fn can_process_op_f_x_1e() {
        let mut chip8 = Chip8::initialize();
        let i_value = 0xF; 
        let v_index = 3;
        let v_value = 0xE;        

        chip8.registers.i = i_value;
        chip8.registers.v[v_index] = v_value;

        chip8.execute_op_code(0xF31E);

        assert_eq!(i_value + (v_value as u16), chip8.registers.i);
    }
}

