use std::collections::HashSet;

use snow_core::geometry::Rect;
use snow_core::simulation::{Snowfall, WindowSurface};

#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    pub id: u64,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub flags: u32,
}

pub const FLAG_MAXIMIZED: u32 = 1;
pub const FLAG_FULLSCREEN: u32 = 2;
pub const FLAG_COVER_ONLY: u32 = 4;
pub const FLAG_ROUNDED: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Monitor {
    pub index: usize,
    pub area: Area,
    pub bounds: Area,
    pub scale: f32,
}

pub fn walls(monitor: &Monitor, all: &[Monitor]) -> (bool, bool) {
    let (area, b) = (monitor.area, monitor.bounds);
    let beside = |x: i32, left_edge: bool| {
        all.iter().any(|other| {
            let o = other.bounds;
            o != b && o.y < b.y + b.height && o.y + o.height > b.y && if left_edge { o.x + o.width == x } else { o.x == x }
        })
    };
    (area.x != b.x || !beside(b.x, true), area.x + area.width != b.x + b.width || !beside(b.x + b.width, false))
}

#[cfg(test)]
mod cover_tests {
    use super::*;

    fn window(id: u64, x: i32, y: i32, width: i32, height: i32) -> Window {
        Window { id, x, y, width, height, flags: 0 }
    }

    #[test]
    fn only_a_topmost_window_over_the_whole_area_covers_it() {
        let area = Area { x: 0, y: 32, width: 1920, height: 1048 };
        let mut desktop = Desktop::new();
        desktop.windows = vec![window(1, 0, 32, 1920, 1048)];
        assert!(desktop.top_covers(&area));
        desktop.windows = vec![window(2, 100, 100, 400, 300), window(1, 0, 32, 1920, 1048)];
        assert!(!desktop.top_covers(&area), "okno nad zmaksymalizowanym ma śnieg na krawędzi");
        desktop.windows = vec![window(3, 2000, 0, 800, 600), window(1, 0, 32, 1920, 1048)];
        assert!(desktop.top_covers(&area), "okno na innym monitorze nie przeszkadza");
        desktop.windows = vec![window(1, 0, 32, 1900, 1048)];
        assert!(!desktop.top_covers(&area));
        desktop.windows = vec![];
        assert!(!desktop.top_covers(&area));
    }
}

#[cfg(test)]
mod walls_tests {
    use super::*;

    fn monitor(index: usize, x: i32, width: i32, dock_left: i32) -> Monitor {
        let bounds = Area { x, y: 0, width, height: 1080 };
        Monitor { index, area: Area { x: x + dock_left, y: 32, width: width - dock_left, height: 1048 }, bounds, scale: 1.0 }
    }

    #[test]
    fn single_monitor_has_both_walls() {
        let m = monitor(0, 0, 1920, 0);
        assert_eq!(walls(&m, &[m.clone()]), (true, true));
    }

    #[test]
    fn neighbours_open_the_shared_edge() {
        let (a, b) = (monitor(0, 0, 1920, 0), monitor(1, 1920, 2560, 0));
        let all = [a.clone(), b.clone()];
        assert_eq!(walls(&a, &all), (true, false));
        assert_eq!(walls(&b, &all), (false, true));
    }

    #[test]
    fn side_dock_closes_the_shared_edge() {
        let (a, b) = (monitor(0, 0, 1920, 0), monitor(1, 1920, 2560, 64));
        assert_eq!(walls(&b, &[a, b.clone()]), (true, true));
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub snowfall: Snowfall,
    pub accumulate: bool,
    pub melt: bool,
    pub snow_on_windows: bool,
    pub over_windows: bool,
    pub calm_fps: u32,
    pub sharp: bool,
    pub halloween: bool,
    pub idle_slowdown: bool,
    pub rest_when_hidden: bool,
    pub battery_slowdown: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            snowfall: Snowfall::Steady,
            accumulate: true,
            melt: true,
            snow_on_windows: true,
            over_windows: true,
            calm_fps: 20,
            sharp: false,
            halloween: true,
            idle_slowdown: false,
            rest_when_hidden: false,
            battery_slowdown: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Power {
    Full,
    Battery,
    Saver,
}

pub struct Desktop {
    pub windows: Vec<Window>,
    pub elsewhere: HashSet<u64>,
    pub generation: u64,
    pub moving: bool,
    pub started_moving: bool,
    pub cursor: Option<(f32, f32)>,
    pub all_covered: bool,
    pub fullscreen: HashSet<usize>,
    pub paused: bool,
    pub idle: bool,
}

impl Desktop {
    pub fn new() -> Desktop {
        Desktop {
            windows: Vec::new(),
            elsewhere: HashSet::new(),
            generation: 0,
            moving: false,
            started_moving: false,
            cursor: None,
            all_covered: false,
            fullscreen: HashSet::new(),
            idle: false,
            paused: false,
        }
    }

    pub fn take_started_moving(&mut self) -> bool {
        std::mem::take(&mut self.started_moving)
    }

    pub fn surfaces(&self, area: &Area, size: (f32, f32)) -> Vec<WindowSurface> {
        let (width, height) = size;
        let sx = width / area.width.max(1) as f32;
        let sy = height / area.height.max(1) as f32;
        let (left, top, right, bottom) = (area.x, area.y, area.x + area.width, area.y + area.height);
        let mut blockers: Vec<Rect> = Vec::new();
        let mut surfaces = Vec::new();
        for window in &self.windows {
            let (wl, wt, wr, wb) = (window.x, window.y, window.x + window.width, window.y + window.height);
            if wr <= left || wl >= right || wb <= top || wt >= bottom {
                continue;
            }
            let frame = Rect {
                x: (wl - left) as f32 * sx,
                y: (bottom - wb) as f32 * sy,
                width: window.width as f32 * sx,
                height: window.height as f32 * sy,
            };
            let room_above = (wt - top) as f32 * sy >= 6.0 && frame.max_y() < height;
            let on_screen = frame.max_x() > 0.0 && frame.min_x() < width && frame.max_y() > 0.0;
            let fills_screen = frame.width * frame.height >= width * height * 0.9;
            let cover_only = window.flags & FLAG_COVER_ONLY != 0;
            if room_above && on_screen && !fills_screen && !cover_only && frame.width >= 80.0 && frame.height >= 40.0 {
                let radius = corner_radius(window).min(frame.width / 2.0).min(frame.height / 2.0);
                let region = Rect { x: frame.x, y: frame.max_y() - radius, width: frame.width, height: 40.0 + radius };
                surfaces.push(WindowSurface {
                    id: window.id as isize,
                    frame,
                    blockers: blockers.iter().filter(|b| intersects(b, &region)).copied().collect(),
                    corner_radius: radius,
                    corner_exponent: 2.0,
                });
            }
            blockers.push(frame);
        }
        surfaces
    }

    pub fn masks(&self, area: &Area, size: (f32, f32)) -> Vec<[f32; 4]> {
        let sx = size.0 / area.width.max(1) as f32;
        let sy = size.1 / area.height.max(1) as f32;
        let bottom = area.y + area.height;
        self.windows
            .iter()
            .filter(|w| w.x + w.width > area.x && w.x < area.x + area.width && w.y + w.height > area.y && w.y < bottom)
            .map(|w| {
                [
                    (w.x - area.x) as f32 * sx,
                    (bottom - (w.y + w.height)) as f32 * sy,
                    (w.x + w.width - area.x) as f32 * sx,
                    (bottom - w.y) as f32 * sy,
                ]
            })
            .take(32)
            .collect()
    }

    pub fn covered(&self, monitor: usize) -> bool {
        self.paused || self.all_covered || self.fullscreen.contains(&monitor)
    }

    pub fn top_covers(&self, area: &Area) -> bool {
        let top = self.windows.iter().find(|w| {
            w.x < area.x + area.width && w.x + w.width > area.x && w.y < area.y + area.height && w.y + w.height > area.y
        });
        top.is_some_and(|w| w.x <= area.x && w.y <= area.y && w.x + w.width >= area.x + area.width && w.y + w.height >= area.y + area.height)
    }

    pub fn is_elsewhere(&self, id: u64) -> bool {
        self.elsewhere.contains(&id)
    }
}

fn corner_radius(window: &Window) -> f32 {
    if window.flags & (FLAG_MAXIMIZED | FLAG_FULLSCREEN) != 0 {
        0.0
    } else if window.flags & FLAG_ROUNDED != 0 {
        12.0
    } else {
        8.0
    }
}

fn intersects(a: &Rect, b: &Rect) -> bool {
    a.min_x() < b.max_x() && b.min_x() < a.max_x() && a.min_y() < b.max_y() && b.min_y() < a.max_y()
}

pub fn parse_snowfall(text: &str) -> Option<Snowfall> {
    Snowfall::ALL.iter().copied().find(|s| s.title().eq_ignore_ascii_case(text))
}
