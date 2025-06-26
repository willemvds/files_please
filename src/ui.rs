use std::collections;
use std::error;
use std::fs;
use std::mem;
use std::path;

//use crate::task;
use crate::directory;

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
                    //eprintln!("Making new texture for text {}", text);
                    let surface = font.render(text).blended(colour)?;
                    let (e, texture) = texture_manager
                        .create_from_surface(entity_manager, surface)
                        .unwrap();
                    self.insert(String::from(text), colour, size, e);
                    texture
                }
            } else {
                //eprintln!("Making new texture for text {}", text);
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
    scrollbar: pixels::Color,
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
            scrollbar: pixels::Color::RGB(180, 180, 180),
            byte: pixels::Color::RGB(100, 160, 20),
            kilo: pixels::Color::RGB(140, 160, 20),
            mega: pixels::Color::RGB(180, 160, 20),
            giga: pixels::Color::RGB(240, 160, 20),
        }
    }
}

pub struct DirectoryViewEntry {
    entry: directory::Entry,
    selected: bool,
}

pub struct DirectoryView {
    dir: path::PathBuf,
    entries: Vec<DirectoryViewEntry>,
    draw_region: render::FRect,
    line_height: f32,
    scroll_index: usize,
    selected_index: Option<usize>,
}

impl From<&directory::Entries> for DirectoryView {
    fn from(de: &directory::Entries) -> DirectoryView {
        let mut dv = DirectoryView::new(de.absolute_path.clone());
        for entry in de.entries.iter() {
            dv.entries.push(DirectoryViewEntry {
                entry: entry.clone(),
                selected: false,
            });
        }

        dv
    }
}

impl DirectoryView {
    pub fn new(abs_path: path::PathBuf) -> DirectoryView {
        DirectoryView {
            dir: abs_path,
            entries: vec![],
            draw_region: render::FRect::new(0.0, 0.0, 0.0, 0.0),
            line_height: 24.0,
            scroll_index: 0,
            selected_index: None,
        }
    }

    fn num_lines(view_height: f32, line_height: f32) -> usize {
        ((view_height - (2.0 * line_height)) / line_height).round() as usize
    }

    pub fn set_draw_region(&mut self, region: render::FRect) {
        self.draw_region = region;
    }

    pub fn top(&mut self) {
        if self.entries.len() > 0 {
            self.selected_index = Some(0);
            self.scroll_index = 0;
        }
    }

    pub fn bottom(&mut self) {
        if self.entries.len() > 0 {
            self.selected_index = Some(self.entries.len() - 1);
            self.down(0);
        }
    }

    pub fn up(&mut self, distance: usize) {
        if let Some(current) = self.selected_index {
            let delta = if current < distance {
                current
            } else {
                distance
            };
            let hover_index = current - delta;
            self.selected_index = Some(hover_index);

            if hover_index < self.scroll_index {
                self.scroll_index = hover_index;
            }
        }
    }

    pub fn down(&mut self, distance: usize) {
        if let Some(current) = self.selected_index {
            let delta = if current + distance < self.entries.len() {
                distance
            } else {
                self.entries.len() - current - 1
            };
            let hover_index = current + delta;
            self.selected_index = Some(hover_index);

            let num_lines = DirectoryView::num_lines(self.draw_region.h, self.line_height);
            if hover_index >= self.scroll_index + num_lines {
                self.scroll_index = hover_index - num_lines + 1;
            }
        }
    }

    // TODO(@willemvds): Need to distinguish between select and cursor/hover.
    pub fn toggle_select(&mut self) {
        if let Some(current) = self.selected_index {
            self.entries[current].selected = !self.entries[current].selected;
        }
    }

    pub fn hovered_entry(&self) -> Option<directory::Entry> {
        if let Some(current) = self.selected_index {
            return Some(self.entries[current].entry.clone());
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
        active: bool,
        font: &sdl3::ttf::Font,
    ) -> Result<(), Box<dyn error::Error>> {
        canvas.set_draw_color(if active { theme.active } else { theme.inactive });
        let _ = canvas.fill_rect(self.draw_region);

        let num_lines = DirectoryView::num_lines(self.draw_region.h, self.line_height);

        let first = self.scroll_index;
        let mut last = first + num_lines + 1;
        if last > self.entries.len() {
            last = self.entries.len()
        }

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
                self.draw_region.x + padding,
                self.draw_region.y + padding + next,
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

        for idx in first..last {
            let entry = &self.entries[idx];

            if let Some(selected_index) = self.selected_index {
                if active && selected_index == idx {
                    canvas.set_draw_color(theme.cursor);
                    let _ = canvas.fill_rect(render::FRect::new(
                        self.draw_region.x,
                        self.draw_region.y + padding + next,
                        self.draw_region.w,
                        24.0,
                    ));
                }
            }

            if let Some(text) = entry.entry.name.clone().into_os_string().to_str() {
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
                        self.draw_region.x + file_size_width,
                        self.draw_region.y + padding + next,
                        select_width,
                        24.0,
                    ));
                }

                let _ = text_manager.render(
                    entity_manager,
                    texture_manager,
                    canvas,
                    font,
                    if entry.entry.kind == directory::EntryKind::Dir {
                        dir_icon
                    } else {
                        file_icon
                    },
                    theme.text,
                    18,
                    self.draw_region.x + file_size_width + select_width * 2.0 + padding,
                    self.draw_region.y + padding + next,
                );

                let _ = text_manager.render(
                    entity_manager,
                    texture_manager,
                    canvas,
                    font,
                    text,
                    theme.text,
                    18,
                    self.draw_region.x
                        + file_size_width
                        + select_width * 2.0
                        + icon_width
                        + padding,
                    self.draw_region.y + padding + next,
                );

                next += 24.0;
            }

            // scrollbar
            if active && self.entries.len() > num_lines {
                let scrollbar_tick =
                    (self.draw_region.h - 28.0 - padding) / self.entries.len() as f32;
                let scrollbar_y = padding + 28.0 + (self.scroll_index as f32 * scrollbar_tick);
                let scrollbar_height = num_lines as f32 * scrollbar_tick;
                let scrollbar_width = 5.0;

                canvas.set_draw_color(theme.scrollbar);
                let _ = canvas.fill_rect(render::FRect::new(
                    self.draw_region.x + self.draw_region.w - scrollbar_width,
                    scrollbar_y,
                    scrollbar_width,
                    scrollbar_height,
                ));
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

enum DirectoryViewState {
    Active,
    Inactive(DirectoryView),
}

pub struct UI<'ui> {
    left_directory_views: collections::HashMap<path::PathBuf, DirectoryViewState>,
    right_directory_views: collections::HashMap<path::PathBuf, DirectoryViewState>,
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
        left_entries: directory::Entries,
        right_entries: directory::Entries,
    ) -> UI<'ui> {
        let mut left_directory_views = collections::HashMap::new();
        left_directory_views.insert(
            left_entries.absolute_path.clone(),
            DirectoryViewState::Active,
        );
        let mut right_directory_views = collections::HashMap::new();
        right_directory_views.insert(
            right_entries.absolute_path.clone(),
            DirectoryViewState::Active,
        );

        let mut ui = UI {
            left_directory_views: left_directory_views,
            right_directory_views: right_directory_views,
            theme: Theme::default(),
            entity_manager: EntityManager::new(),
            text_manager: TextManager::new(),
            texture_manager: TextureManager::new(texture_creator),
            active: Side::Left,
            font: font,
            lhs: DirectoryView::from(&left_entries),
            rhs: DirectoryView::from(&right_entries),
            tv: TasksView::new(),
        };
        ui.lhs.selected_index = Some(0);
        ui.rhs.selected_index = Some(0);
        ui
    }

    pub fn update_dir_entries(&mut self, de: directory::Entries) {
        let left_e = self.left_directory_views.entry(de.absolute_path.clone());
        left_e.or_insert(DirectoryViewState::Inactive(DirectoryView::from(&de)));

        let right_e = self.right_directory_views.entry(de.absolute_path.clone());
        right_e.or_insert(DirectoryViewState::Inactive(DirectoryView::from(&de)));
    }

    pub fn up(&mut self, distance: usize) {
        match self.active {
            Side::Left => self.lhs.up(distance),
            Side::Right => self.rhs.up(distance),
        }
    }

    pub fn down(&mut self, distance: usize) {
        match self.active {
            Side::Left => self.lhs.down(distance),
            Side::Right => self.rhs.down(distance),
        }
    }

    pub fn top(&mut self) {
        match self.active {
            Side::Left => self.lhs.top(),
            Side::Right => self.rhs.top(),
        }
    }

    pub fn bottom(&mut self) {
        match self.active {
            Side::Left => self.lhs.bottom(),
            Side::Right => self.rhs.bottom(),
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

    pub fn hovered_entry(&self) -> Option<directory::Entry> {
        let dv = self.active_directory_view();
        dv.hovered_entry()
    }

    pub fn show_dir(&mut self, abs_path: path::PathBuf, selected_entry: path::PathBuf) {
        let side_directory_views = match self.active {
            Side::Left => &mut self.left_directory_views,
            Side::Right => &mut self.right_directory_views,
        };
        let side = match self.active {
            Side::Left => &mut self.lhs,
            Side::Right => &mut self.rhs,
        };
        if let Some(dvs) = side_directory_views.get(&abs_path) {
            match dvs {
                DirectoryViewState::Active => {
                    // this means we received new dir entries for the currently shown dv
                    eprintln!("Active? for {}", abs_path.display());
                }
                DirectoryViewState::Inactive(_) => {
                    eprintln!("Inactive? for {}", abs_path.display());
                    if let Some(prev) = side_directory_views.remove(&abs_path) {
                        side_directory_views.insert(abs_path.clone(), DirectoryViewState::Active);
                        match prev {
                            DirectoryViewState::Inactive(active_dv) => {
                                let mut selected_index = active_dv.selected_index;
                                if active_dv.selected_index.is_none() {
                                    for (idx, entry) in active_dv.entries.iter().enumerate() {
                                        if entry.entry.name == selected_entry {
                                            selected_index = Some(idx);
                                            break;
                                        }
                                    }
                                }
                                if selected_index.is_none() && side.entries.len() > 0 {
                                    selected_index = Some(0)
                                }

                                let old_dv = mem::replace(side, active_dv);
                                side.selected_index = selected_index;
                                side_directory_views.insert(
                                    old_dv.dir.clone(),
                                    DirectoryViewState::Inactive(old_dv),
                                );
                            }
                            _ => {
                                eprintln!("This should not happen ....");
                            }
                        }
                    }
                }
            }
        } else {
            eprintln!(
                "Trying to show dir without entries...{}",
                abs_path.display()
            );
        }
    }

    pub fn next(&mut self) {}

    pub fn prev(&mut self) {}

    pub fn render(&mut self, canvas: &mut render::Canvas<video::Window>) {
        canvas.clear();

        let (w, h) = canvas.window().size();
        let ww = w as f32;
        let hh = h as f32;

        let left_region = render::FRect::new(0.0, 0.0, ww / 2.0, hh - 200.0);
        self.lhs.set_draw_region(left_region);
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
            left_active,
            self.font,
        );
        let right_region = render::FRect::new(ww / 2.0, 0.0, ww / 2.0, hh - 200.0);
        self.rhs.set_draw_region(right_region);
        let _ = self.rhs.render(
            canvas,
            &self.theme,
            &mut self.entity_manager,
            &mut self.text_manager,
            &mut self.texture_manager,
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
