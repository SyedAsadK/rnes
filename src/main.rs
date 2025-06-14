use comp::bus::Bus;
// use comp::cpu::Mem;
use comp::cpu::CPU;
use comp::rom::Rom;
use comp::trace::*;
use sdl2::pixels::PixelFormatEnum;

pub mod comp;
fn main() {
    // init sdl2

    let window_width = 1280;
    let window_height = 720;

    //
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Snake game", (window_width) as u32, (window_height) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(10.0, 10.0).unwrap();

    let creator = canvas.texture_creator();
    let texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 32, 32)
        .unwrap();

    let bytes: Vec<u8> = std::fs::read("nestest.nes").unwrap();
    let rom = Rom::new(&bytes).unwrap();

    let bus = Bus::new(rom);
    let mut cpu = CPU::new(bus);
    cpu.reset();
    cpu.pc = 0xc000;

    cpu.run_with_callback(
        move |cpu| {
            println!("{}", trace(cpu));
            // cpu.mem_write(0xfe, rng.gen_range(1, 16));
            // if read_screen_state(cpu, &mut screen_state) {
            //     texture.update(None, &screen_state, 32 * 3).unwrap();
            //     canvas.copy(&texture, None, None).unwrap();
            //     canvas.present();
        }, // ::std::thread::sleep(std::time::Duration::new(0, 70_000));
    );
}
