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
    cursor: pixels::Color,
    header: pixels::Color,
    selected: pixels::Color,
    byte: pixels::Color,
    kilo: pixels::Color,
    mega: pixels::Color,
    giga: pixels::Color,
}

impl Theme {
    pub fn default() -> Theme {
        Theme {
            active: pixels::Color::RGB(90, 90, 90),
            inactive: pixels::Color::RGB(70, 70, 70),
            tasks: pixels::Color::RGB(45, 200, 155),
            text: pixels::Color::RGB(22, 255, 44),
            cursor: pixels::Color::RGB(70, 50, 122),
            header: pixels::Color::RGB(250, 250, 250),
            selected: pixels::Color::RGB(250, 120, 0),
            byte: pixels::Color::RGB(100, 160, 20),
            kilo: pixels::Color::RGB(140, 160, 20),
            mega: pixels::Color::RGB(180, 160, 20),
            giga: pixels::Color::RGB(240, 160, 20),
        }
    }
}

#[derive(PartialEq)]
enum DirectoryViewEntryKind {
    Dir,
    File,
}

struct DirectoryViewEntry {
    selected: bool,
    kind: DirectoryViewEntryKind,
    name: path::PathBuf,
}

impl DirectoryViewEntry {
    fn new(
        kind: DirectoryViewEntryKind,
        name: path::PathBuf,
        selected: bool,
    ) -> DirectoryViewEntry {
        DirectoryViewEntry {
            kind,
            name,
            selected,
        }
    }
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
        self.entries.push(DirectoryViewEntry::new(
            DirectoryViewEntryKind::File,
            name,
            true,
        ));
        if self.selected_index.is_none() {
            self.selected_index = Some(0)
        }
    }

    pub fn push_dir(&mut self, name: path::PathBuf) {
        self.entries.push(DirectoryViewEntry::new(
            DirectoryViewEntryKind::Dir,
            name,
            true,
        ));

        if self.selected_index.is_none() {
            self.selected_index = Some(0)
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

    // TODO(@willemvds): Need to distinguish between select and cursor/hover.
    pub fn toggle_select(&mut self) {
        if let Some(current) = self.selected_index {
            self.entries[current].selected = !self.entries[current].selected;
        }
    }

    pub fn hovered_entry(&self) -> Option<path::PathBuf> {
        if let Some(current) = self.selected_index {
            return Some(self.entries[current].name.clone());
        }
        None
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

        let icon_width = 20.0;
        let dir_icon = "\u{f4d3}";
        let file_icon = " ";
        let select_width = 4.0;
        let file_size_width = 0.0;

        for (idx, entry) in self.entries.iter().enumerate() {
            if let Some(selected_index) = self.selected_index {
                if active && selected_index == idx {
                    canvas.set_draw_color(theme.cursor);
                    let _ = canvas.fill_rect(render::FRect::new(
                        region.x,
                        region.y + padding + next,
                        region.w,
                        24.0,
                    ));
                }
            }

            if let Some(text) = entry.name.clone().into_os_string().to_str() {
                //let _ = text_manager.render(
                //    entity_manager,
                //    texture_manager,
                //    canvas,
                //    font,
                //    "4MB",
                //    theme.giga,
                //    18,
                //    region.x + padding + 10.0,
                //    region.y + padding + next,
                //);

                if active && entry.selected {
                    canvas.set_draw_color(theme.selected);
                    let _ = canvas.fill_rect(render::FRect::new(
                        region.x + file_size_width,
                        region.y + padding + next,
                        select_width,
                        24.0,
                    ));
                }

                let _ = text_manager.render(
                    entity_manager,
                    texture_manager,
                    canvas,
                    font,
                    if entry.kind == DirectoryViewEntryKind::Dir {
                        dir_icon
                    } else {
                        file_icon
                    },
                    theme.text,
                    18,
                    region.x + file_size_width + select_width * 2.0 + padding,
                    region.y + padding + next,
                );

                let _ = text_manager.render(
                    entity_manager,
                    texture_manager,
                    canvas,
                    font,
                    text,
                    theme.text,
                    18,
                    region.x + file_size_width + select_width * 2.0 + icon_width + padding,
                    region.y + padding + next,
                );

                next += 24.0;
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

    pub fn update_dir_view(&mut self, dv: DirectoryView) {
        match self.active {
            Side::Left => self.lhs = dv,
            Side::Right => self.rhs = dv,
        }
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

    pub fn toggle_select(&mut self) {
        match self.active {
            Side::Left => self.lhs.toggle_select(),
            Side::Right => self.rhs.toggle_select(),
        }
    }

    pub fn active_directory_view(&self) -> &DirectoryView {
        match self.active {
            Side::Left => &self.lhs,
            Side::Right => &self.rhs,
        }
    }

    pub fn active_dir_path(&self) -> path::PathBuf {
        self.active_directory_view().dir.clone()
    }

    pub fn hovered_entry(&self) -> Option<path::PathBuf> {
        let dv = self.active_directory_view();
        dv.hovered_entry()
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
