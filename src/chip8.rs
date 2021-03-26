use rand::Rng;
const FONT_SPRITE_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80
    ];

pub struct Chip8 {
    v: [u8; 16], // general registers
    pc: u16, // program counter
    i: u16, // register I
    sound_timer: u8,
    delay_timer: u8,
    memory: [u8; 4096],
    opcode: u16,
    gfx: [[u8; 32]; 64],
    stack: Vec<u16>,
    keys: [bool; 16],
    draw: bool,
    playing_sound: bool
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            v: [0; 16], // general registers
            pc: 0x0200,
            i: 0, // register I
            sound_timer: 0,
            delay_timer: 0,
            memory: [0; 4096],
            opcode: 0,
            gfx: [[0; 32]; 64],
            stack: Vec::new(),
            keys: [false; 16],
            draw: false,
            playing_sound: false
        }
    }

    pub fn set_key(&mut self, k: usize, pressed: bool) {
        self.keys[k] = pressed;
    }

    pub fn get_gfx(&mut self) -> &[[u8; 32]; 64] {
        return &self.gfx;
    }

    pub fn get_draw(&mut self) -> bool {
        return self.draw;
    }

    pub fn set_draw(&mut self, draw: bool) {
        self.draw = draw;
    }

    pub fn load_rom(&mut self, data: Vec<u8>) {
        for j in 0..=(data.len()-1) {
            self.memory[0x0200 + j] = data[j];
        }
    }

    pub fn load_fonts(&mut self) {
        for j in 0..50 {
            self.memory[j] = FONT_SPRITE_DATA[j];
        }
    }

    pub fn emulate_frame(&mut self) {
        for _j in 0..9 {
            self.emulate_cycle();
        }
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.playing_sound = true;
            self.sound_timer -= 1;
        } else {
            self.playing_sound = false;
        }
    }

    pub fn emulate_cycle(&mut self) {
        self.fetch_opcode();
        self.execute_opcode();    
    }

    pub fn is_playing_sound(&mut self) -> bool {
        return self.playing_sound;
    }

    pub fn fetch_opcode(&mut self) {
        self.opcode = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16);
    }
    pub fn execute_opcode(&mut self) {
        let x = ((self.opcode >> 8) & 0xF) as usize;
        let y = ((self.opcode >> 4) & 0xF) as usize;
        let n = (self.opcode & 0xFF) as u8;
        //println!("{:04x}", self.opcode);
        match self.opcode & 0xF000 {
            0x0000 => match self.opcode {
                0x00E0 => self.cls(), // Clear screen
                0x00EE => self.rts(), // Return from a subroutine
                _ => () // (0x0NNN) (Ignored). Execute machine language subroutine at address NNN
            },
            0x1000 => self.jump(self.opcode & 0x0FFF), // (0x1NNN) Jump to address NNN
            0x2000 => self.call(self.opcode & 0x0FFF), // (0x2NNN) Execute subroutine starting at address NNN
            0x3000 => self.ske(x, n), // (0x3XNN) Skip the following instruction if the value of register VX equals NN
            0x4000 => self.skne(x, n), // (0x4XNN) Skip the following instruction if the value of register VX does not equal NN
            0x5000 => match self.opcode & 0xF00F {
                0x5000 => self.skre(x, y), // (0x5XY0) Skip the following instruction if the value of register VX is equal to the value of register VY
                _ => () // Undefined
            },
            0x6000 => self.load(x, n), // (0x6XNN) Store number NN in register VX
            0x7000 => self.add(x, n), // (0x7XNN) Add the value NN to register VX
            0x8000 => match self.opcode & 0xF00F {
                0x8000 => self.r#move(x, y), // (0x8XY0) Store the value of register VY in register VX
                0x8001 => self.or(x, y), // (0x8XY1) Set VX to VX OR VY
                0x8002 => self.and(x, y), // (0x8XY2) Set VX to VX AND VY
                0x8003 => self.xor(x, y), // (0x8XY3) Set VX to VX XOR VY
                0x8004 => self.addr(x, y), // (0x8XY4) Add the value of register VY to register VX. Set VF to 01 if a carry occurs; set VF to 00 if a carry does not occur
                0x8005 => self.sub(x, y), // (0x8XY5) Subtract the value of register VY from register VX. Set VF to 00 if a borrow occurs; set VF to 01 if a borrow does not occur
                0x8006 => self.shr(x, y), // (0x8XY6) Store the value of register VY shifted right one bit in register VX. Set register VF to the least significant bit prior to the shift. VY is unchanged
                0x8007 => self.subn(x, y), // (0x8XY7) Set register VX to the value of VY minus VX. Set VF to 00 if a borrow occurs. Set VF to 01 if a borrow does not occur
                0x800E => self.shl(x, y), // (0x8XYE) Store the value of register VY shifted left one bit in register VX. Set register VF to the most significant bit prior to the shift. VY is unchanged
                _ => () // Undefined
            },
            0x9000 => match self.opcode & 0xF00F {
                0x9000 => self.skrne(x, y), // (0x9XY0) Skip the following instruction if the value of register VX is not equal to the value of register VY
                _ => () // Undefined
            },
            0xA000 => self.loadi(self.opcode & 0x0FFF), // (0xANNN) Store memory address NNN in register I
            0xB000 => self.jump0(self.opcode & 0x0FFF), // (0xBNNN) Jump to address NNN + V0
            0xC000 => self.rand(x, n), // (0xCXNN) Set VX to a random number with a mask of NN
            0xD000 => self.draw(x, y, (self.opcode & 0x000F) as usize), // (0xDXYN) Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I. Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
            0xE000 => match self.opcode & 0xF0FF {
                0xE09E => self.skpr(x), // (0xEX9E) Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed
                0xE0A1 => self.skup(x), // (0xEXA1) Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed
                _ => (), // Undefined
            }
            0xF000 => match self.opcode & 0xF0FF {
                0xF007 => self.moved(x), // (0xFX07) Store the current value of the delay timer in register VX
                0xF00A => self.keyd(x), // (0xFX0A) Wait for a keypress and store the result in register VX
                0xF015 => self.loadd(x), // (0xFX15) Set the delay timer to the value of register VX
                0xF018 => self.loads(x), // (0xFX18) Set the sound timer to the value of register VX
                0xF01E => self.addi(x), // (0xFX1E) Add the value stored in register VX in register I
                0xF029 => self.ldspr(x), // (0xFX29) Set register I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register VX
                0xF033 => self.bcd(x), // (0xFX33) Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I+1, and I+2
                0xF055 => self.stor(x), // (0xFX55) Store the values of registers V0 to VX inclusive in memory starting at address I. I is set to I+X+1 after operation
                0xF065 => self.read(x), // (0xFX65) Fill registers V0 to VX inclusive with the values stored in memory starting at address I. I is set to I+X+1 after operation
                _ => (), // Undefined
            }
            _ => (), // Undefined
        }
    }

    // (0x7XNN) Add the value NN to register VX
    pub fn add(&mut self, x: usize, n: u8) {
        self.v[x] = self.v[x].wrapping_add(n);
        self.pc += 2;
    }

    pub fn addi(&mut self, x: usize) {
        self.i += self.v[x] as u16;
        self.pc += 2;
    }

    pub fn addr(&mut self, x: usize, y:usize) {
        if (0xFF - self.v[x]) < self.v[y] {
            self.v[x] = self.v[x].wrapping_add(self.v[y]);
            self.v[0xF] = 1;
        } else {
            self.v[x] = self.v[x].wrapping_add(self.v[y]);
            self.v[0xF] = 0;
        }
        self.pc += 2;
    }

    // (0x8XY2) Set VX to VX AND VY
    pub fn and(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
        self.pc += 2;
    }

    // Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I+1, and I+2
    pub fn bcd(&mut self, x: usize) {
        self.memory[self.i as usize] = self.v[x] / 100;
        self.memory[(self.i + 1) as usize] = (self.v[x] / 10) % 10;
        self.memory[(self.i + 2) as usize] = self.v[x] % 10;
        self.pc += 2;
    }

    pub fn call(&mut self, addr: u16) {
        self.stack.push(self.pc);
        self.pc = addr;
    }

    // Clear screen
    pub fn cls(&mut self) {
        for x in 0..=63 {
            for y in 0..=31 {
                self.gfx[x][y] = 0;
            }
        }
        self.pc += 2;
    }
    
    pub fn draw(&mut self, x: usize, y: usize, n: usize) {
        let mut draw_flag : bool = false;
        let col = (self.v[x] % 64) as usize;
        let row = (self.v[y] % 32) as usize;
        for j in 0..=n-1 {
            if j + row >= 32 {
                break;
            }
            for k in 0..=7 {
                if k + col >= 64 {
                    break;
                } else {
                    let on : u8 = (self.memory[(self.i as usize) + j] >> (7-k)) & 1;
                    if on & self.gfx[k + col][j + row] == 1 {
                        draw_flag = true;
                    }
                    self.gfx[k + col][j + row] ^= on;
                }
            }
        }

        if draw_flag {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.draw = true;
        self.pc += 2;
    }

    // (0x1NNN) Jump to address NNN
    pub fn jump(&mut self, addr: u16) {
        self.pc = addr;
    }

    pub fn jump0(&mut self, addr: u16) {
        self.pc = addr + (self.v[0] as u16);
    }

    pub fn keyd(&mut self, x: usize) {
        for key in 0..0xF {
            if self.keys[key] {
                self.v[x] = key as u8;
                self.pc += 2;
                return;
            }
        }
    }

    pub fn ldspr(&mut self, x: usize) {
        self.i = (self.v[x] as u16) * 5;
        self.pc += 2;
    }

    pub fn load(&mut self, x: usize, n: u8) {
        self.v[x] = n;
        self.pc += 2;
    }

    pub fn loadd(&mut self, x: usize) {
        self.delay_timer = self.v[x];
        self.pc += 2;
    }

    pub fn loadi(&mut self, addr: u16) {
        self.i = addr;
        self.pc += 2;
    }

    pub fn loads(&mut self, x: usize) {
        self.sound_timer = self.v[x];
        self.pc += 2;
    }

    pub fn r#move(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
        self.pc += 2;
    }

    pub fn moved(&mut self, x: usize) {
        self.v[x] = self.delay_timer;
        self.pc += 2;
    }

    pub fn or(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
        self.pc += 2;
    }

    pub fn rand(&mut self, x: usize, n: u8) {
        let mut rng = rand::thread_rng();
        let r : u8 = rng.gen();
        self.v[x] = r & n;
        self.pc += 2;
    }

    pub fn read(&mut self, x: usize) {
        for j in 0..=x {
            self.v[j] = self.memory[(self.i as usize) + j];
        }
        self.i += (x + 1) as u16;
        self.pc += 2;
    }

    // Return from a subroutine
    pub fn rts(&mut self) {
        let s = self.stack.pop();
        self.pc = match s {
            None => self.pc,
            _=> s.unwrap()
        };
        self.pc += 2;
    }

    pub fn shl(&mut self, x: usize, y: usize) {
        let b = (self.v[y] & 0x80) >> 7;
        self.v[x] = self.v[y] << 1;
        self.v[0xF] = b;
        self.pc += 2;
    }

    pub fn shr(&mut self, x: usize, y: usize) {
        self.v[0xF] = self.v[y] & 1;
        self.v[x] = self.v[y] >> 1;
        self.pc += 2;
    }

    pub fn ske(&mut self, x: usize, n: u8) {
        if self.v[x] == n {
            self.pc += 2;
        }
        self.pc += 2;
    }

    pub fn skne(&mut self, x: usize, n: u8) {
        if self.v[x] != n {
            self.pc += 2;
        }
        self.pc += 2;
    }

    pub fn skpr(&mut self, x: usize) {
        if self.keys[self.v[x] as usize] {
            self.pc += 2
        }
        self.pc += 2;
    }

    pub fn skup(&mut self, x: usize) {
        if !self.keys[self.v[x] as usize] {
            self.pc += 2
        }
        self.pc += 2;
    }

    pub fn skre(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
        self.pc += 2;
    }

    pub fn skrne(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
        self.pc += 2;
    }

    pub fn stor(&mut self, x: usize) {
        for j in 0..=x {
            self.memory[(self.i as usize) + j] = self.v[j];
        }
        self.i += (x + 1) as u16;
        self.pc += 2;
    }

    pub fn sub(&mut self, x: usize, y:usize) {
        if self.v[x] >= self.v[y] {
            self.v[x] = self.v[x].wrapping_sub(self.v[y]);
            self.v[0xF] = 1;
        } else {
            self.v[x] = self.v[x].wrapping_sub(self.v[y]);
            self.v[0xF] = 0;
        }
        self.pc += 2;
    }

    
    pub fn subn(&mut self, x: usize, y: usize) {
        if self.v[y] >= self.v[x] {
            self.v[x] = self.v[y].wrapping_sub(self.v[x]);
            self.v[0xF] = 1;
        } else {
            self.v[x] = self.v[y].wrapping_sub(self.v[x]);
            self.v[0xF] = 0;
        }
        self.pc += 2;
    }

    pub fn xor(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
        self.pc += 2;
    }
}