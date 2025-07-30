use std::env::args;

use comp::bus::Bus;
// use comp::cpu::Mem;
use comp::cpu::CPU;
use comp::rom::Rom;
use comp::tiles::tile;
use comp::trace::*;
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

        let window_width = 1280;
        let window_height = 720;

        //
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Rnes", (window_width) as u32, (window_height) as u32)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        canvas.set_scale(10.0, 10.0).unwrap();

        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(PixelFormatEnum::RGB24, 32, 32)
            .unwrap();

        let mut frame = Frame::new();

        let rom_name = args.get(1).unwrap();
        let bytes: Vec<u8> = std::fs::read(rom_name).unwrap();
        let rom = Rom::new(&bytes).unwrap();

        let bus = Bus::new(rom, move |ppu: &NesPPU| {
            render::render(ppu, &mut frame);
            texture.update(None, &frame.data, 256 * 3).unwrap();
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
