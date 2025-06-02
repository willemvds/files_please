use std::path;
use std::process;

mod ui;

extern crate sdl3;
use sdl3::event;
use sdl3::keyboard;

const EXIT_CODE_OK: u8 = 0;
const EXIT_CODE_SDL_ERROR: u8 = 1;

fn files_please_gui() -> Result<(), process::ExitCode> {
    let sdl_context = sdl3::init().map_err(|err| {
        eprintln!("SDL3 Init err={}", err);
        process::ExitCode::from(EXIT_CODE_SDL_ERROR)
    })?;

    let sdl_video = sdl_context.video().map_err(|err| {
        eprintln!("SDL3 Video err={}", err);
        process::ExitCode::from(EXIT_CODE_SDL_ERROR)
    })?;

    let sdl_ttf = sdl3::ttf::init().map_err(|err| {
        eprintln!("SDL3 TTF err={}", err);
        process::ExitCode::from(EXIT_CODE_SDL_ERROR)
    })?;

    let window = sdl_video
        .window("Files, Please!", 1200, 800)
        .maximized()
        .build()
        .map_err(|err| {
            eprintln!("SDL3 Window err={}", err);
            process::ExitCode::from(EXIT_CODE_SDL_ERROR)
        })?;

    let display = window.get_display().map_err(|err| {
        eprintln!("SDL3 Display err={}", err);
        process::ExitCode::from(EXIT_CODE_SDL_ERROR)
    })?;

    // let display_mode = display.get_mode();
    let font = sdl_ttf
        .load_font("SauceCodeProNerdFontMono-Regular.ttf", 18.0)
        .map_err(|err| {
            eprintln!("SDL3 Font err={}", err);
            process::ExitCode::from(EXIT_CODE_SDL_ERROR)
        })?;

    let mut canvas = window.into_canvas();
    let mut event_pump = sdl_context.event_pump().map_err(|err| {
        eprintln!("SDL Event err={}", err);
        process::ExitCode::from(EXIT_CODE_SDL_ERROR)
    })?;

    let mut gui = ui::UI::new(&font);
    let mut left = ui::DirectoryView::new();
    left.push(path::PathBuf::from("Not"));
    left.push(path::PathBuf::from("Real"));
    left.push(path::PathBuf::from("Yet"));
    gui.left_dir_view(left);
    let mut right = ui::DirectoryView::new();
    right.push(path::PathBuf::from("\u{e6ae}"));
    gui.right_dir_view(right);

    loop {
        for ev in event_pump.poll_iter() {
            match ev {
                event::Event::Quit { .. }
                | event::Event::KeyDown {
                    keycode: Some(keyboard::Keycode::Escape),
                    ..
                } => return Ok(()),
                _ => {}
            }
        }

        gui.render(&mut canvas);
    }
}

fn main() -> process::ExitCode {
    match files_please_gui() {
        Ok(()) => process::ExitCode::from(EXIT_CODE_OK),
        Err(exit_code) => exit_code,
    }
}
