mod chip8;
use crate::chip8::Chip8;

extern crate sdl2;
extern crate tinyfiledialogs;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::surface::Surface;
use sdl2::render::Texture;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};
use crate::tinyfiledialogs::open_file_dialog;

use sdl2::audio::{AudioCallback, AudioSpecDesired};

const FRAMERATE : u32 = 60;

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
 
pub fn main() {
    let filter : Option<(&[&str], &str)> = Some((&["*.ch8", "*.rom"], "CHIP-8 binaries (.ch8, .rom)"));
    let rom_path = open_file_dialog(
        "Open rom",
        "./",
        filter
    ).expect("Failed to open file!");
    let binary = match std::fs::read(rom_path) {
        Ok(s) => s,
        Err(_) => Vec::new()
    };
    let mut machine_state : Chip8 = Chip8::new();
    machine_state.load_fonts();
    machine_state.load_rom(binary);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1),
        samples: None,
    };

    let mut device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        SquareWave {
            phase_inc: 220.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.0,
        }
    }).unwrap();
    device.resume();
 
    let window = video_subsystem.window("CHIP-8 Interpreter", 800, 600)
        .position_centered()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let texture_creator = canvas.texture_creator();
    let surface = Surface::new(64, 32, PixelFormatEnum::RGB24).unwrap();
    let mut texture = Texture::from_surface(&surface, &texture_creator).unwrap();
    let mut pixel_data : [u8; 2048 * 4] = [0; 2048 * 4];
    let mut event_pump = sdl_context.event_pump().unwrap();
    let frametime = Duration::new(0, 1_000_000_000u32 / FRAMERATE);
    'running: loop {
        let instant = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode : Some(Keycode::X), .. } => {machine_state.set_key(0x0, true)},
                Event::KeyDown { keycode : Some(Keycode::Num1), .. } => {machine_state.set_key(0x1, true)},
                Event::KeyDown { keycode : Some(Keycode::Num2), .. } => {machine_state.set_key(0x2, true)},
                Event::KeyDown { keycode : Some(Keycode::Num3), .. } => {machine_state.set_key(0x3, true)},
                Event::KeyDown { keycode : Some(Keycode::Q), .. } => {machine_state.set_key(0x4, true)},
                Event::KeyDown { keycode : Some(Keycode::W), .. } => {machine_state.set_key(0x5, true)},
                Event::KeyDown { keycode : Some(Keycode::E), .. } => {machine_state.set_key(0x6, true)},
                Event::KeyDown { keycode : Some(Keycode::A), .. } => {machine_state.set_key(0x7, true)},
                Event::KeyDown { keycode : Some(Keycode::S), .. } => {machine_state.set_key(0x8, true)},
                Event::KeyDown { keycode : Some(Keycode::D), .. } => {machine_state.set_key(0x9, true)},
                Event::KeyDown { keycode : Some(Keycode::Z), .. } => {machine_state.set_key(0xa, true)},
                Event::KeyDown { keycode : Some(Keycode::C), .. } => {machine_state.set_key(0xb, true)},
                Event::KeyDown { keycode : Some(Keycode::Num4), .. } => {machine_state.set_key(0xc, true)},
                Event::KeyDown { keycode : Some(Keycode::R), .. } => {machine_state.set_key(0xd, true)},
                Event::KeyDown { keycode : Some(Keycode::F), .. } => {machine_state.set_key(0xe, true)},
                Event::KeyDown { keycode : Some(Keycode::V), .. } => {machine_state.set_key(0xf, true)},
                Event::KeyUp { keycode : Some(Keycode::X), .. } => {machine_state.set_key(0x0, false)},
                Event::KeyUp { keycode : Some(Keycode::Num1), .. } => {machine_state.set_key(0x1, false)},
                Event::KeyUp { keycode : Some(Keycode::Num2), .. } => {machine_state.set_key(0x2, false)},
                Event::KeyUp { keycode : Some(Keycode::Num3), .. } => {machine_state.set_key(0x3, false)},
                Event::KeyUp { keycode : Some(Keycode::Q), .. } => {machine_state.set_key(0x4, false)},
                Event::KeyUp { keycode : Some(Keycode::W), .. } => {machine_state.set_key(0x5, false)},
                Event::KeyUp { keycode : Some(Keycode::E), .. } => {machine_state.set_key(0x6, false)},
                Event::KeyUp { keycode : Some(Keycode::A), .. } => {machine_state.set_key(0x7, false)},
                Event::KeyUp { keycode : Some(Keycode::S), .. } => {machine_state.set_key(0x8, false)},
                Event::KeyUp { keycode : Some(Keycode::D), .. } => {machine_state.set_key(0x9, false)},
                Event::KeyUp { keycode : Some(Keycode::Z), .. } => {machine_state.set_key(0xa, false)},
                Event::KeyUp { keycode : Some(Keycode::C), .. } => {machine_state.set_key(0xb, false)},
                Event::KeyUp { keycode : Some(Keycode::Num4), .. } => {machine_state.set_key(0xc, false)},
                Event::KeyUp { keycode : Some(Keycode::R), .. } => {machine_state.set_key(0xd, false)},
                Event::KeyUp { keycode : Some(Keycode::F), .. } => {machine_state.set_key(0xe, false)},
                Event::KeyUp { keycode : Some(Keycode::V), .. } => {machine_state.set_key(0xf, false)},
                _ => {}       }
        }

        machine_state.emulate_frame();
        if machine_state.get_draw() {
            for i in 0..2048 {
                let x = i % 64;
                let y = i / 64;
                pixel_data[4 * i] = 0xFF * machine_state.get_gfx()[x][y];
                pixel_data[4 * i + 1] = 0xFF * machine_state.get_gfx()[x][y];
                pixel_data[4 * i + 2] = 0xFF * machine_state.get_gfx()[x][y];
                pixel_data[4 * i + 3] = 0xFF;
            }
            texture.update(None, &pixel_data, 64 * 4).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            machine_state.set_draw(false);
            canvas.present();
        }

        if machine_state.is_playing_sound() {
            let mut lock = device.lock(); 
            lock.volume = 0.10;
        } else {
            let mut lock = device.lock(); 
            lock.volume = 0.0;
        }
        
        match frametime.checked_sub(instant.elapsed()) {
            None => (),
            Some(t) => ::std::thread::sleep(t)
        };        
    }
}