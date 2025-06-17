use std::collections;
use std::error;
use std::path;

//use crate::task;

extern crate sdl3;
use sdl3::pixels;
use sdl3::render;
use sdl3::surface;
use sdl3::ttf;
use sdl3::video;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
struct Entity(u64);

struct EntityManager {
    next: Entity,
}

impl EntityManager {
    fn new() -> EntityManager {
        EntityManager { next: Entity(1) }
    }

    fn next(&mut self) -> Entity {
        let next = self.next;
        self.next = Entity(self.next.0 + 1);
        next
    }
}

struct TextureManager<'t> {
    creator: &'static render::TextureCreator<video::WindowContext>,
    cache: collections::HashMap<Entity, render::Texture<'t>>,
}

impl<'t> TextureManager<'t> {
    fn new(creator: &'static render::TextureCreator<video::WindowContext>) -> TextureManager<'t> {
        TextureManager {
            creator: creator,
            cache: collections::HashMap::new(),
        }
    }

    fn create_from_surface(
        &mut self,
        entity_manager: &mut EntityManager,
        surface: surface::Surface,
    ) -> Result<(Entity, &render::Texture<'t>), Box<dyn error::Error>> {
        let tex = self.creator.create_texture_from_surface(surface)?;
        let entity = entity_manager.next();
        self.cache.insert(entity, tex);
        Ok((entity, self.cache.get(&entity).unwrap()))
    }

    fn get(&self, tid: Entity) -> Option<&render::Texture<'t>> {
        self.cache.get(&tid)
    }
}

struct TextManager {
    repo: collections::HashMap<(String, pixels::Color, usize), Entity>,
}

impl TextManager {
    fn new() -> TextManager {
        TextManager {
            repo: collections::HashMap::new(),
        }
    }

    fn get(&self, text: String, colour: pixels::Color, size: usize) -> Option<Entity> {
        self.repo.get(&(text, colour, size)).copied()
    }

    fn insert(&mut self, text: String, colour: pixels::Color, size: usize, entity: Entity) {
        self.repo.insert((text, colour, size), entity);
    }

    fn render(
        &mut self,
        entity_manager: &mut EntityManager,
        texture_manager: &mut TextureManager,
        canvas: &mut render::Canvas<video::Window>,
        font: &ttf::Font,
        text: &str,
        colour: pixels::Color,
        size: usize,
        x: f32,
        y: f32,
    ) -> Result<(), Box<dyn error::Error>> {
        let tex = {
            if let Some(entity) = self.get(String::from(text), colour, size) {
                if let Some(texture) = texture_manager.get(entity) {
                    texture
                } else {
                    eprintln!("Making new texture for text {}", text);
                    let surface = font.render(text).blended(colour)?;
                    let (e, texture) = texture_manager
                        .create_from_surface(entity_manager, surface)
                        .unwrap();
                    self.insert(String::from(text), colour, size, e);
                    texture
                }
            } else {
                eprintln!("Making new texture for text {}", text);
                let surface = font.render(text).blended(colour)?;
                let (e, texture) = texture_manager
                    .create_from_surface(entity_manager, surface)
                    .unwrap();
                self.insert(String::from(text), colour, size, e);
                texture
            }
        };

        let target = render::FRect::new(x, y, tex.width() as f32, tex.height() as f32);
        let _ = canvas.copy(&tex, None, Some(target));
        Ok(())
    }
}

pub struct Theme {
    active: pixels::Color,
    inactive: pixels::Color,
    tasks: pixels::Color,
    text: pixels::Color,
    selected: pixels::Color,
    header: pixels::Color,
}

impl Theme {
    pub fn default() -> Theme {
        Theme {
            active: pixels::Color::RGB(90, 90, 90),
            inactive: pixels::Color::RGB(70, 70, 70),
            tasks: pixels::Color::RGB(45, 200, 155),
            text: pixels::Color::RGB(22, 255, 44),
            selected: pixels::Color::RGB(70, 50, 122),
            header: pixels::Color::RGB(250, 250, 250),
        }
    }
}

enum DirectoryViewEntryKind {
    Dir,
    File,
}

struct DirectoryViewEntry {
    kind: DirectoryViewEntryKind,
    name: path::PathBuf,
}

pub struct DirectoryView {
    dir: path::PathBuf,
    entries: Vec<DirectoryViewEntry>,
    selected_index: Option<usize>,
}

impl DirectoryView {
    pub fn new(abs_path: path::PathBuf) -> DirectoryView {
        DirectoryView {
            dir: abs_path,
            entries: vec![],
            selected_index: None,
        }
    }

    pub fn push_file(&mut self, name: path::PathBuf) {
        self.entries.push(DirectoryViewEntry {
            kind: DirectoryViewEntryKind::File,
            name: name,
        })
    }

    pub fn push_dir(&mut self, name: path::PathBuf) {
        self.entries.push(DirectoryViewEntry {
            kind: DirectoryViewEntryKind::Dir,
            name: name,
        })
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
        entity_manager: &mut EntityManager,
        text_manager: &mut TextManager,
        texture_manager: &mut TextureManager,
        region: render::FRect,
        active: bool,
        font: &sdl3::ttf::Font,
    ) -> Result<(), Box<dyn error::Error>> {
        canvas.set_draw_color(if active { theme.active } else { theme.inactive });
        let _ = canvas.fill_rect(region);

        let padding = 5.0;
        let mut next = 0.0;

        if let Some(text) = self.dir.clone().into_os_string().to_str() {
            let _ = text_manager.render(
                entity_manager,
                texture_manager,
                canvas,
                font,
                text,
                theme.header,
                18,
                region.x + padding,
                region.y + padding + next,
            );
            next += 28.0;
        };
        //let surface = font.render(text).blended(theme.header)?;
        //let tc = canvas.texture_creator();
        //let tex = tc.create_texture_from_surface(surface)?;

        for (idx, entry) in self.entries.iter().enumerate() {
            if let Some(selected_index) = self.selected_index {
                if active && selected_index == idx {
                    canvas.set_draw_color(theme.selected);
                    let _ = canvas.fill_rect(render::FRect::new(
                        region.x,
                        region.y + padding + next + 1.0,
                        region.w,
                        22.0,
                    ));
                }
            }
            if let Some(text) = entry.name.clone().into_os_string().to_str() {
                let _ = text_manager.render(
                    entity_manager,
                    texture_manager,
                    canvas,
                    font,
                    text,
                    theme.text,
                    18,
                    region.x + padding,
                    region.y + padding + next,
                );

                next += 20.0;
            }
        }

        Ok(())
    }
}

pub struct TaskView {
    //task: task::Task,
}

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
    entity_manager: EntityManager,
    text_manager: TextManager,
    texture_manager: TextureManager<'ui>,
    active: Side,
    font: &'ui sdl3::ttf::Font<'ui, 'ui>,
    lhs: DirectoryView,
    rhs: DirectoryView,
    tv: TasksView,
}

impl<'ui> UI<'ui> {
    pub fn new(
        texture_creator: &'static render::TextureCreator<video::WindowContext>,
        font: &'ui sdl3::ttf::Font,
        mut left_dir_view: DirectoryView,
        mut right_dir_view: DirectoryView,
    ) -> UI<'ui> {
        left_dir_view.select(0);
        right_dir_view.select(0);
        UI {
            theme: Theme::default(),
            entity_manager: EntityManager::new(),
            text_manager: TextManager::new(),
            texture_manager: TextureManager::new(texture_creator),
            active: Side::Left,
            font: font,
            lhs: left_dir_view,
            rhs: right_dir_view,
            tv: TasksView::new(),
        }
    }

    pub fn left_dir_view(&mut self, dv: DirectoryView) {
        self.lhs = dv;
        self.lhs.select(0);
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

    pub fn toggle_side(&mut self) {
        match self.active {
            Side::Left => self.active = Side::Right,
            Side::Right => self.active = Side::Left,
        }
    }

    pub fn next(&mut self) {}

    pub fn prev(&mut self) {}

    pub fn render(&mut self, canvas: &mut render::Canvas<video::Window>) {
        canvas.clear();

        let (w, h) = canvas.window().size();
        let ww = w as f32;
        let hh = h as f32;

        let left_region = render::FRect::new(0.0, 0.0, ww / 2.0, hh);
        let left_active = match self.active {
            Side::Left => true,
            Side::Right => false,
        };

        let right_active = match self.active {
            Side::Left => false,
            Side::Right => true,
        };
        let _ = self.lhs.render(
            canvas,
            &self.theme,
            &mut self.entity_manager,
            &mut self.text_manager,
            &mut self.texture_manager,
            left_region,
            left_active,
            self.font,
        );
        let right_region = render::FRect::new(ww / 2.0, 0.0, ww / 2.0, hh);
        let _ = self.rhs.render(
            canvas,
            &self.theme,
            &mut self.entity_manager,
            &mut self.text_manager,
            &mut self.texture_manager,
            right_region,
            right_active,
            self.font,
        );

        let tasks_region = render::FRect::new(0.0, hh - 200.0, ww, 200.0);
        let _ = self
            .tv
            .render(canvas, tasks_region, self.theme.tasks, self.font);

        canvas.present();
    }
}
