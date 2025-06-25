use std::collections;
use std::env;
use std::fs;
use std::path;
use std::process;
use std::thread;
use std::time;

//mod task;
mod directory;
mod ui;

extern crate sdl3;
use sdl3::event;
use sdl3::keyboard;

const EXIT_CODE_OK: u8 = 0;
const EXIT_CODE_SDL_ERROR: u8 = 1;

enum Action {
    Up,
    Down,
    JumpUp,
    JumpDown,
    Top,
    Bottom,
    Next,
    Prev,
    ToggleSide,
    ToggleSelect,
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
        (keyboard::Keycode::Home, Action::Top),
        (keyboard::Keycode::End, Action::Bottom),
        (keyboard::Keycode::PageUp, Action::JumpUp),
        (keyboard::Keycode::PageDown, Action::JumpDown),
        (keyboard::Keycode::Left, Action::Prev),
        (keyboard::Keycode::Right, Action::Next),
        (keyboard::Keycode::Tab, Action::ToggleSide),
        (keyboard::Keycode::Space, Action::ToggleSelect),
        (keyboard::Keycode::Escape, Action::Quit),
    ]);

    let mut dir_path = env::current_dir().unwrap_or(path::PathBuf::from("."));

    let read_dir_it = fs::read_dir(&dir_path).map_err(|err| {
        eprintln!("Failed to read current working directory {}", err);
        process::ExitCode::from(2)
    })?;
    let de = directory::Entries::new(dir_path.clone(), read_dir_it);

    let mut gui = ui::UI::new(texture_creator, &font, de.clone(), de.clone());

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
                            Action::Up => gui.up(1),
                            Action::Down => gui.down(1),
                            Action::Top => gui.top(),
                            Action::Bottom => gui.bottom(),
                            Action::JumpUp => gui.up(10),
                            Action::JumpDown => gui.down(10),
                            Action::Next => {
                                if let Some(hovered_entry) = gui.hovered_entry() {
                                    eprintln!(
                                        "next on hovered entry {}",
                                        hovered_entry.name.display()
                                    );
                                    dir_path = gui.active_dir_path();
                                    dir_path.push(hovered_entry.name);

                                    if let Ok(read_dir_it) = fs::read_dir(&dir_path) {
                                        let de =
                                            directory::Entries::new(dir_path.clone(), read_dir_it);
                                        gui.update_dir_entries(de);
                                    }
                                }
                                gui.show_dir(dir_path.clone());
                            }
                            Action::Prev => {
                                dir_path = gui.active_dir_path();
                                dir_path.pop();
                                if let Ok(read_dir_it) = fs::read_dir(&dir_path) {
                                    let de = directory::Entries::new(dir_path.clone(), read_dir_it);
                                    gui.update_dir_entries(de);
                                }
                                gui.show_dir(dir_path.clone());
                            }
                            Action::ToggleSide => gui.toggle_side(),
                            Action::ToggleSelect => gui.toggle_select(),
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
