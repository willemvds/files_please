use std::error;
use std::path;

extern crate sdl3;
use sdl3::pixels;
use sdl3::render;
use sdl3::video;

pub struct DirectoryView {
    entries: Vec<path::PathBuf>,
}

impl DirectoryView {
    pub fn new() -> DirectoryView {
        DirectoryView { entries: vec![] }
    }

    pub fn push(&mut self, entry: path::PathBuf) {
        self.entries.push(entry)
    }

    fn render(
        &self,
        canvas: &mut render::Canvas<video::Window>,
        region: render::FRect,
        colour: pixels::Color,
        font: &sdl3::ttf::Font,
    ) -> Result<(), Box<dyn error::Error>> {
        canvas.set_draw_color(colour);
        let _ = canvas.fill_rect(region);

        let padding = 5.0;
        let mut next = 0.0;
        for entry in self.entries.iter() {
            if let Some(text) = entry.clone().into_os_string().to_str() {
                let surface = font.render(text).blended(pixels::Color::RGB(22, 255, 44))?;
                let tc = canvas.texture_creator();
                let tex = tc.create_texture_from_surface(surface)?;
                let target = render::FRect::new(
                    region.x + padding,
                    region.y + padding + next,
                    tex.width() as f32,
                    tex.height() as f32,
                );
                let _ = canvas.copy(&tex, None, Some(target));
                next += 20.0;
            }
        }

        Ok(())
    }
}

pub struct TaskView {}

pub struct TasksView {
    tasks: Vec<TaskView>,
}

impl TasksView {
    pub fn new() -> TasksView {
        TasksView { tasks: vec![] }
    }

    fn render(
        &self,
        canvas: &mut render::Canvas<video::Window>,
        region: render::FRect,
        colour: pixels::Color,
        font: &sdl3::ttf::Font,
    ) -> Result<(), Box<dyn error::Error>> {
        canvas.set_draw_color(colour);
        let _ = canvas.fill_rect(region);

        Ok(())
    }
}

pub struct UI<'ui> {
    font: &'ui sdl3::ttf::Font<'ui, 'ui>,
    lhs: DirectoryView,
    rhs: DirectoryView,
    tv: TasksView,
}

impl<'ui> UI<'ui> {
    pub fn new(font: &'ui sdl3::ttf::Font) -> UI<'ui> {
        UI {
            font: font,
            lhs: DirectoryView::new(),
            rhs: DirectoryView::new(),
            tv: TasksView::new(),
        }
    }

    pub fn left_dir_view(&mut self, dv: DirectoryView) {
        self.lhs = dv
    }

    pub fn right_dir_view(&mut self, dv: DirectoryView) {
        self.rhs = dv
    }

    pub fn render(&self, canvas: &mut render::Canvas<video::Window>) {
        canvas.set_draw_color(pixels::Color::RGB(0, 255, 255));
        canvas.clear();

        let (w, h) = canvas.window().size();
        let ww = w as f32;
        let hh = h as f32;

        let left_region = render::FRect::new(0.0, 0.0, ww / 2.0, hh);
        let _ = self.lhs.render(
            canvas,
            left_region,
            pixels::Color::RGB(90, 90, 90),
            self.font,
        );
        let right_region = render::FRect::new(ww / 2.0, 0.0, ww / 2.0, hh);
        let _ = self.rhs.render(
            canvas,
            right_region,
            pixels::Color::RGB(50, 50, 50),
            self.font,
        );

        let tasks_region = render::FRect::new(0.0, hh - 200.0, ww, 200.0);
        let _ = self.tv.render(
            canvas,
            tasks_region,
            pixels::Color::RGB(45, 200, 155),
            self.font,
        );

        canvas.present();
    }
}
