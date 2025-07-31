use std::collections::HashMap;
use std::env::args;

use comp::bus::Bus;
// use comp::cpu::Mem;
use comp::controller::{Controller, ControllerButtons};
use comp::cpu::CPU;
use comp::rom::Rom;
use comp::tiles::tile;
// use comp::trace::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

use crate::comp::ppu::NesPPU;
use crate::comp::render;
use crate::comp::render::frame::Frame;

pub mod comp;
fn main() {
    let args: Vec<String> = args().collect();
    if args.get(1).unwrap() == "--tiles" {
        tile();
    } else {
        // init sdl2

        //
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Rnes", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        canvas.set_scale(5.0, 5.0).unwrap();

        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
            .unwrap();

        let mut frame = Frame::new();

        let mut key_map = HashMap::new();
        key_map.insert(Keycode::Down, ControllerButtons::DOWN);
        key_map.insert(Keycode::Up, ControllerButtons::UP);
        key_map.insert(Keycode::Right, ControllerButtons::RIGHT);
        key_map.insert(Keycode::Left, ControllerButtons::LEFT);
        key_map.insert(Keycode::Space, ControllerButtons::SELECT);
        key_map.insert(Keycode::Return, ControllerButtons::START);
        key_map.insert(Keycode::C, ControllerButtons::BUTTON_A);
        key_map.insert(Keycode::X, ControllerButtons::BUTTON_B);

        let rom_name = args.get(1).unwrap();
        let bytes: Vec<u8> = std::fs::read(rom_name).unwrap();
        let rom = Rom::new(&bytes).unwrap();

        let bus = Bus::new(rom, move |ppu: &NesPPU, cont: &mut Controller| {
            render::render(ppu, &mut frame);
            texture
                .update(None, &frame.data, 256 * 3)
                .expect("Problem here");
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),
                    Event::KeyDown {
                        keycode: Some(Keycode::Q),
                        ..
                    } => std::process::exit(0),
                    Event::KeyDown { keycode, .. } => {
                        if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                            cont.set_button_pressed_status(*key, true);
                        }
                    }
                    Event::KeyUp { keycode, .. } => {
                        if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                            cont.set_button_pressed_status(*key, false);
                        }
                    }
                    _ => { /* do nothing */ }
                }
            }
        });
        let mut cpu = CPU::new(bus);
        cpu.reset();
        cpu.run();

        //     cpu.run_with_callback(
        //         move |cpu| {
        //             println!("{}", trace(cpu));
        //             // cpu.mem_write(0xfe, rng.gen_range(1, 16));
        //             // if read_screen_state(cpu, &mut screen_state) {
        //             //     texture.update(None, &screen_state, 32 * 3).unwrap();
        //             //     canvas.copy(&texture, None, None).unwrap();
        //             //     canvas.present();
        //         }, // ::std::thread::sleep(std::time::Duration::new(0, 70_000));
        //     );
    }
}
