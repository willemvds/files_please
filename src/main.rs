use std::collections;
use std::path;
use std::process;
use std::thread;
use std::time;

//mod task;
mod ui;

extern crate sdl3;
use sdl3::event;
use sdl3::keyboard;

const EXIT_CODE_OK: u8 = 0;
const EXIT_CODE_SDL_ERROR: u8 = 1;

enum Action {
    Up,
    Down,
    Next,
    Prev,
    ToggleSide,
    Quit,
}

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
        .resizable()
        .borderless()
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
    let texture_creator = Box::leak(Box::new(canvas.texture_creator()));

    let mut event_pump = sdl_context.event_pump().map_err(|err| {
        eprintln!("SDL Event err={}", err);
        process::ExitCode::from(EXIT_CODE_SDL_ERROR)
    })?;

    let keybinds = collections::HashMap::from([
        (keyboard::Keycode::Up, Action::Up),
        (keyboard::Keycode::Down, Action::Down),
        (keyboard::Keycode::Left, Action::Next),
        (keyboard::Keycode::Right, Action::Prev),
        (keyboard::Keycode::Tab, Action::ToggleSide),
        (keyboard::Keycode::Escape, Action::Quit),
    ]);

    let mut left = ui::DirectoryView::new(path::PathBuf::from("/home"));
    left.push_dir(path::PathBuf::from("\u{f4d3} Not"));
    left.push_dir(path::PathBuf::from("Real"));
    left.push_file(path::PathBuf::from("Yet"));
    left.push_file(path::PathBuf::from("."));
    left.push_file(path::PathBuf::from(".."));
    left.push_file(path::PathBuf::from("..."));
    //gui.left_dir_view(left);
    let mut right = ui::DirectoryView::new(path::PathBuf::from("/root"));
    right.push_dir(path::PathBuf::from("\u{e6ae}"));
    right.push_dir(path::PathBuf::from("\u{e6ae}"));
    right.push_dir(path::PathBuf::from("\u{e6ae}"));
    right.push_dir(path::PathBuf::from("\u{e6ae}"));
    right.push_file(path::PathBuf::from("\u{e6ae}"));
    right.push_file(path::PathBuf::from("\u{e6ae}"));
    right.push_file(path::PathBuf::from("\u{e6ae}"));
    //gui.right_dir_view(right);

    //let l2 = left.clone();

    let mut gui = ui::UI::new(texture_creator, &font, left, right);

    loop {
        for ev in event_pump.poll_iter() {
            match ev {
                event::Event::Quit { .. } => return Ok(()),
                event::Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(action) = keybinds.get(&keycode) {
                        match action {
                            Action::Quit => return Ok(()),
                            Action::Up => gui.up(),
                            Action::Down => gui.down(),
                            Action::Next => gui.next(),
                            Action::Prev => gui.prev(),
                            Action::ToggleSide => gui.toggle_side(),
                            _ => {}
                        }
                        //gui.up();
                    }
                }
                _ => {}
            }
        }

        gui.render(&mut canvas);
        thread::sleep(time::Duration::from_micros(2000));
    }
}

fn main() -> process::ExitCode {
    match files_please_gui() {
        Ok(()) => process::ExitCode::from(EXIT_CODE_OK),
        Err(exit_code) => exit_code,
    }
}
