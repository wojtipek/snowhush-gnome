use std::collections::HashSet;
use std::time::{Duration, Instant};

use snow_core::geometry::V2;
use snow_core::simulation::{Pace, SnowSimulation, WindowId, WindowSurface};

use crate::damage::Damage;
use crate::desktop::{Desktop, Monitor, Power, Settings};
use crate::egl::{EGLSurface, Egl};
use crate::render::{Ambient, Renderer};
use crate::wl::{Wayland, WlWindow};

pub const AWAY: V2 = V2::new(-10_000.0, -10_000.0);
const REST: Duration = Duration::from_secs(1);
const FRAME_WAIT: Duration = Duration::from_secs(1);
const IDLE_FPS: u32 = 10;
const BATTERY_CALM_FPS: u32 = 15;
const BATTERY_FASTEST_FPS: u32 = 30;
pub const APP_ID: &str = "org.snowhush.Overlay";

pub struct Screen {
    pub monitor: Monitor,
    pub simulation: SnowSimulation,
    pub window: WlWindow,
    surface: EGLSurface,
    pub next_due: Instant,
    last_step: Option<Instant>,
    idle_frames: u32,
    surfaces: Vec<WindowSurface>,
    windows_seen: u64,
    windows_changed_at: Instant,
    covered: bool,
    cleared: bool,
    damage: Damage,
    pub damaged: f32,
    lantern: V2,
    waiting_since: Option<Instant>,
    postponed_from: Option<Instant>,
    hidden: bool,
    must_draw: bool,
}

impl Screen {
    pub fn new(wayland: &Wayland, egl: &Egl, monitor: Monitor, settings: &Settings, now: Instant) -> Result<Screen, String> {
        let size = (monitor.area.width, monitor.area.height);
        let mut window = wayland.create_window(&title(monitor.index), APP_ID, size);
        window.set_fallback_size(size);
        let surface = egl.create_window_surface(window.egl_window)?;
        egl.make_current(surface)?;
        egl.no_vsync();
        let mut simulation = SnowSimulation::new(size.0 as f32, size.1 as f32, settings.snowfall);
        simulation.thaws = settings.melt;
        simulation.accumulates = settings.accumulate;
        crate::log!("monitor {}: obszar {:?}, skala {}", monitor.index, monitor.area, monitor.scale);
        let mut screen = Screen {
            monitor,
            simulation,
            window,
            surface,
            next_due: now,
            last_step: None,
            idle_frames: 0,
            surfaces: Vec::new(),
            windows_seen: 0,
            windows_changed_at: now,
            covered: false,
            cleared: false,
            damage: Damage::new(),
            damaged: 0.0,
            lantern: AWAY,
            waiting_since: None,
            postponed_from: None,
            hidden: false,
            must_draw: false,
        };
        screen.apply_sharpness(settings.sharp);
        Ok(screen)
    }

    pub fn update_monitor(&mut self, monitor: Monitor, settings: &Settings) {
        if monitor == self.monitor {
            return;
        }
        let size = (monitor.area.width, monitor.area.height);
        self.window.set_fallback_size(size);
        crate::log!("monitor {}: nowy obszar {:?}, skala {}", monitor.index, monitor.area, monitor.scale);
        self.monitor = monitor;
        self.apply_sharpness(settings.sharp);
    }

    pub fn apply_sharpness(&mut self, sharp: bool) {
        let scale = if sharp { self.monitor.scale.ceil().max(1.0) as i32 } else { 1 };
        self.window.set_buffer_scale(scale);
        self.damage.invalidate();
        self.wake(Instant::now());
    }

    pub fn handle_configure(&mut self) {
        if let Some((width, height)) = self.window.take_configure() {
            let (w, h) = (width as f32, height as f32);
            if (w - self.simulation.width()).abs() > 0.5 || (h - self.simulation.height()).abs() > 0.5 {
                crate::log!("monitor {}: okno {}×{}", self.monitor.index, width, height);
                self.simulation.resize(w, h);
                self.surfaces.clear();
                self.windows_seen = 0;
            }
            self.damage.invalidate();
            self.must_draw = true;
            self.waiting_since = None;
            self.postponed_from = None;
            self.wake(Instant::now());
        }
    }

    pub fn destroy(self, wayland: &Wayland, egl: &Egl) {
        egl.destroy_surface(self.surface);
        wayland.destroy_window(self.window);
    }

    pub fn check_frame_callback(&mut self) {
        if self.waiting_since.is_some() && !self.window.frame_pending() {
            self.waiting_since = None;
            if let Some(planned) = self.postponed_from.take() {
                self.next_due = planned;
            }
        }
    }

    pub fn wake(&mut self, now: Instant) {
        self.next_due = self.next_due.min(now);
    }

    pub fn resume(&mut self) {
        self.last_step = None;
        self.next_due = Instant::now();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn frame(
        &mut self,
        renderer: &mut Renderer,
        egl: &Egl,
        desktop: &Desktop,
        settings: &Settings,
        power: Power,
        (light, night): ([f32; 3], f32),
        time: f32,
        now: Instant,
    ) -> Result<bool, String> {
        if !self.window.configured {
            self.next_due = now + Duration::from_millis(100);
            return Ok(false);
        }
        let hidden = settings.rest_when_hidden && !settings.over_windows && desktop.top_covers(&self.monitor.area);
        if hidden != self.hidden {
            crate::log!("monitor {}: {}", self.monitor.index, if hidden { "śnieg zasłonięty oknem" } else { "śnieg znów widoczny" });
            self.hidden = hidden;
        }
        let covered = desktop.covered(self.monitor.index) || hidden;
        if covered != self.covered {
            crate::log!("monitor {}: {}", self.monitor.index, if covered { "zasłonięty" } else { "odsłonięty" });
            self.covered = covered;
            if !covered {
                self.resume();
            }
        }
        if covered {
            if !self.cleared || std::mem::take(&mut self.must_draw) {
                egl.make_current(self.surface)?;
                renderer.clear(self.window.buffer_size());
                egl.swap(self.surface)?;
                self.cleared = true;
                self.damage.invalidate();
            }
            self.next_due = now + Duration::from_millis(250);
            return Ok(false);
        }

        if let Some(since) = self.waiting_since {
            if now.duration_since(since) < FRAME_WAIT {
                self.postponed_from = Some(self.postponed_from.map_or(self.next_due, |p| p.min(self.next_due)));
                self.next_due = since + FRAME_WAIT;
                return Ok(false);
            }
            self.waiting_since = None;
            self.postponed_from = None;
        }

        let elapsed = self.last_step.map_or(1.0 / 60.0, |last| now.duration_since(last).as_secs_f32().min(0.25));
        let substeps = (elapsed / 0.05).ceil().max(1.0) as usize;
        let dt = elapsed / substeps as f32;
        self.last_step = Some(now);
        if settings.snow_on_windows {
            self.follow(desktop, now);
        }
        let cursor = desktop.cursor.map_or(AWAY, |(x, y)| self.local(x, y));
        let mut changed = false;
        for _ in 0..substeps {
            changed |= self.simulation.step(dt, cursor);
        }
        let (width, height) = (self.simulation.width(), self.simulation.height());
        let reach = crate::render::LANTERN_REACH;
        let near = |p: V2| p.x > -reach && p.x < width + reach && p.y > -reach && p.y < height + reach;
        let lantern_moved = night > 0.0 && (cursor - self.lantern).length() > 0.5 && (near(cursor) || near(self.lantern));
        self.lantern = cursor;
        changed |= lantern_moved || std::mem::take(&mut self.must_draw);

        let mut fastest: u32 = 60;
        let mut calm = if power == Power::Saver { settings.calm_fps.min(24) } else { settings.calm_fps };
        if settings.battery_slowdown && power != Power::Full {
            calm = calm.min(BATTERY_CALM_FPS);
            fastest = fastest.min(BATTERY_FASTEST_FPS);
        }
        let mut rate = match self.simulation.pace() {
            Pace::Calm if !lantern_moved => calm,
            _ => fastest.max(calm),
        };
        if settings.idle_slowdown && desktop.idle {
            rate = rate.min(IDLE_FPS);
        }
        self.next_due = now + Duration::from_secs_f64(1.0 / rate as f64);
        if self.simulation.is_empty() {
            self.next_due = now + REST;
        }
        self.idle_frames = if changed { 0 } else { self.idle_frames + 1 };
        let visible = self.simulation.has_snow() || self.simulation.sprite_count() > 0;
        if !(changed || (self.simulation.has_snow() && self.idle_frames.is_multiple_of(2))) {
            if !visible && !self.cleared {
                egl.make_current(self.surface)?;
                renderer.clear(self.window.buffer_size());
                egl.swap(self.surface)?;
                self.cleared = true;
                self.damage.invalidate();
            }
            return Ok(false);
        }
        self.cleared = false;
        egl.make_current(self.surface)?;
        let ambient = Ambient { light, shadow_tint: [0.0; 4], night, lantern: cursor };
        let masks = if settings.over_windows {
            Vec::new()
        } else {
            desktop.masks(&self.monitor.area, (self.simulation.width(), self.simulation.height()))
        };
        let size = self.window.buffer_size();
        let scale = self.window.buffer_scale() as f32;
        let drawn = renderer.draw(&self.simulation, size, scale, ambient, time, &masks)?;
        let rects = if drawn.extras {
            self.damage.invalidate();
            None
        } else {
            self.damage.compute(size, scale, renderer.sprites(drawn.sprites), &drawn.piles)
        };
        self.window.request_frame();
        egl.swap_damaged(self.surface, rects.as_deref())?;
        self.waiting_since = Some(now);
        self.damaged = self.damage.last_fraction;
        Ok(true)
    }

    fn follow(&mut self, desktop: &Desktop, now: Instant) {
        if desktop.generation == self.windows_seen {
            return;
        }
        self.windows_seen = desktop.generation;
        let surfaces = desktop.surfaces(&self.monitor.area, (self.simulation.width(), self.simulation.height()));
        if surfaces == self.surfaces {
            return;
        }
        let since = now.duration_since(self.windows_changed_at).as_secs_f32();
        let interval = if desktop.moving || since < 0.1 { since.min(0.25) } else { 1.0 / 60.0 };
        let elsewhere: HashSet<WindowId> = self
            .surfaces
            .iter()
            .map(|surface| surface.id)
            .filter(|&id| !surfaces.iter().any(|surface| surface.id == id) && desktop.is_elsewhere(id as u64))
            .collect();
        self.simulation.update_windows(&surfaces, interval, &HashSet::new(), &elsewhere, false);
        self.surfaces = surfaces;
        self.windows_changed_at = now;
    }

    pub fn drop_window_snow(&mut self) {
        self.simulation.update_windows(&[], 0.1, &HashSet::new(), &HashSet::new(), false);
        self.surfaces.clear();
        self.windows_seen = 0;
    }

    pub fn local(&self, x: f32, y: f32) -> V2 {
        let area = &self.monitor.area;
        let sx = self.simulation.width() / area.width.max(1) as f32;
        let sy = self.simulation.height() / area.height.max(1) as f32;
        V2::new((x - area.x as f32) * sx, ((area.y + area.height) as f32 - y) * sy)
    }
}

pub fn title(index: usize) -> String {
    format!("snowhush-overlay-{index}")
}
