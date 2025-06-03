use std::error;
use std::path;

extern crate sdl3;
use sdl3::pixels;
use sdl3::render;
use sdl3::video;

pub struct Theme {
    active: pixels::Color,
    inactive: pixels::Color,
    tasks: pixels::Color,
    text: pixels::Color,
    selected: pixels::Color,
}

impl Theme {
    pub fn default() -> Theme {
        Theme {
            active: pixels::Color::RGB(90, 90, 90),
            inactive: pixels::Color::RGB(70, 70, 70),
            tasks: pixels::Color::RGB(45, 200, 155),
            text: pixels::Color::RGB(22, 255, 44),
            selected: pixels::Color::RGB(70, 50, 122),
        }
    }
}

pub struct DirectoryView {
    entries: Vec<path::PathBuf>,
    selected_index: Option<usize>,
}

impl DirectoryView {
    pub fn new() -> DirectoryView {
        DirectoryView {
            entries: vec![],
            selected_index: None,
        }
    }

    pub fn push(&mut self, entry: path::PathBuf) {
        self.entries.push(entry)
    }

    pub fn select(&mut self, index: usize) {
        if index < self.entries.len() {
            self.selected_index = Some(index)
        }
    }

    pub fn up(&mut self) {
        if let Some(current) = self.selected_index {
            if current > 0 {
                self.selected_index = Some(current - 1)
            }
        }
    }

    pub fn down(&mut self) {
        if let Some(current) = self.selected_index {
            if current + 1 < self.entries.len() {
                self.selected_index = Some(current + 1)
            }
        }
    }

    fn render(
        &self,
        canvas: &mut render::Canvas<video::Window>,
        theme: &Theme,
        region: render::FRect,
        colour: pixels::Color,
        font: &sdl3::ttf::Font,
    ) -> Result<(), Box<dyn error::Error>> {
        canvas.set_draw_color(colour);
        let _ = canvas.fill_rect(region);

        let padding = 5.0;
        let mut next = 0.0;
        for (idx, entry) in self.entries.iter().enumerate() {
            if let Some(selected_index) = self.selected_index {
                if selected_index == idx {
                    canvas.set_draw_color(theme.selected);
                    let _ = canvas.fill_rect(render::FRect::new(
                        region.x,
                        region.y + padding + next + 1.0,
                        region.w,
                        22.0,
                    ));
                }
            }
            if let Some(text) = entry.clone().into_os_string().to_str() {
                let surface = font.render(text).blended(theme.text)?;
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

enum Side {
    Left,
    Right,
}

pub struct UI<'ui> {
    theme: Theme,
    active: Side,
    font: &'ui sdl3::ttf::Font<'ui, 'ui>,
    lhs: DirectoryView,
    rhs: DirectoryView,
    tv: TasksView,
}

impl<'ui> UI<'ui> {
    pub fn new(font: &'ui sdl3::ttf::Font) -> UI<'ui> {
        UI {
            theme: Theme::default(),
            active: Side::Left,
            font: font,
            lhs: DirectoryView::new(),
            rhs: DirectoryView::new(),
            tv: TasksView::new(),
        }
    }

    pub fn left_dir_view(&mut self, dv: DirectoryView) {
        self.lhs = dv;
        self.lhs.select(1);
    }

    pub fn right_dir_view(&mut self, dv: DirectoryView) {
        self.rhs = dv;
        self.rhs.select(0);
    }

    pub fn up(&mut self) {
        match self.active {
            Side::Left => self.lhs.up(),
            Side::Right => self.rhs.up(),
        }
    }

    pub fn down(&mut self) {
        match self.active {
            Side::Left => self.lhs.down(),
            Side::Right => self.rhs.down(),
        }
    }

    pub fn left(&mut self) {
        self.active = Side::Left
    }

    pub fn right(&mut self) {
        self.active = Side::Right
    }

    pub fn render(&self, canvas: &mut render::Canvas<video::Window>) {
        //canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        let (w, h) = canvas.window().size();
        let ww = w as f32;
        let hh = h as f32;

        let active_colour = pixels::Color::RGB(90, 90, 90);
        let inactive_colour = pixels::Color::RGB(50, 50, 50);

        let left_region = render::FRect::new(0.0, 0.0, ww / 2.0, hh);
        let left_colour = match self.active {
            Side::Left => active_colour,
            Side::Right => inactive_colour,
        };

        let right_colour = match self.active {
            Side::Right => active_colour,
            Side::Left => inactive_colour,
        };
        let _ = self
            .lhs
            .render(canvas, &self.theme, left_region, left_colour, self.font);
        let right_region = render::FRect::new(ww / 2.0, 0.0, ww / 2.0, hh);
        let _ = self
            .rhs
            .render(canvas, &self.theme, right_region, right_colour, self.font);

        let tasks_region = render::FRect::new(0.0, hh - 200.0, ww, 200.0);
        let _ = self
            .tv
            .render(canvas, tasks_region, self.theme.tasks, self.font);

        canvas.present();
    }
}
