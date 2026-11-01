mod damage;
mod desktop;
mod egl;
mod ffi;
mod gl;
mod light;
mod png;
mod protocol;
mod render;
mod screen;
mod wl;

use std::collections::HashSet;
use std::io::Write;
use std::time::{Duration, Instant};

use snow_core::halloween;

use desktop::{Area, Desktop, Monitor, Power, Settings};
use egl::{Egl, Platform};
use protocol::Command;
use render::Renderer;
use screen::Screen;
use wl::Wayland;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        eprintln!("snowhush-engine: {}", format!($($arg)*));
    }};
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = if let Some(i) = args.iter().position(|a| a == "--snapshot") {
        let path = args.get(i + 1).cloned().unwrap_or_else(|| "snapshot.ppm".into());
        let seconds = value(&args, "--seconds").unwrap_or(40.0);
        snapshot(&path, seconds)
    } else if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("snowhush-engine [--standalone [--size SZERxWYS]] | [--snapshot plik.ppm [--seconds N]]");
        Ok(())
    } else {
        let standalone = args.iter().any(|a| a == "--standalone");
        let size = args
            .iter()
            .position(|a| a == "--size")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.split_once('x'))
            .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
            .unwrap_or((1280, 720));
        run(standalone, size)
    };
    if let Err(error) = result {
        log!("błąd: {error}");
        std::process::exit(1);
    }
}

fn value(args: &[String], name: &str) -> Option<f32> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok())
}

struct App {
    wayland: Wayland,
    egl: Egl,
    renderer: Renderer,
    screens: Vec<Screen>,
    desktop: Desktop,
    settings: Settings,
    saver: bool,
    system_power: Power,
    light: [f32; 3],
    season: bool,
    halloween: bool,
    night: f32,
    started: Instant,
    chores_at: Instant,
    light_at: Instant,
    stats: Stats,
    quit: bool,
}

fn run(standalone: bool, size: (i32, i32)) -> Result<(), String> {
    if !standalone {
        ffi::die_with_parent();
    }
    let wayland = Wayland::connect()?;
    let egl = Egl::new(Platform::Wayland(wayland.display()))?;
    egl.make_current(std::ptr::null_mut())?;
    let gl = gl::Gl::load(&gl::lookup_with(|name| egl.get_proc(name), ffi::Library::open(&["libGLESv2.so.2", "libGLESv2.so"]).ok()))?;
    let renderer = Renderer::new(gl)?;
    let now = Instant::now();
    let mut app = App {
        wayland,
        egl,
        renderer,
        screens: Vec::new(),
        desktop: Desktop::new(),
        settings: Settings::default(),
        saver: false,
        system_power: light::power_from_system(),
        light: light::daylight(),
        season: light::is_halloween(),
        halloween: false,
        night: 0.0,
        started: now,
        chores_at: now,
        light_at: now + Duration::from_secs(60),
        stats: Stats::new(now),
        quit: false,
    };
    if standalone {
        let area = Area { x: 0, y: 0, width: size.0, height: size.1 };
        let monitor = Monitor { index: 0, area, bounds: area, scale: 1.0 };
        app.apply(Command::Monitors(vec![monitor]));
    }
    log!("gotowy{}", if standalone { " (tryb samodzielny)" } else { "" });
    let _ = std::io::stdout().write_all(b"ready\n");
    let _ = std::io::stdout().flush();

    let stdin = 0;
    ffi::set_nonblocking(stdin);
    let mut input = Vec::<u8>::new();
    let mut stdin_open = !standalone;
    let wayland_fd = app.wayland.fd();

    while !app.quit {
        app.wayland.prepare_read()?;
        let now = Instant::now();
        let due = app.screens.iter().map(|s| s.next_due).min();
        let timeout = match due {
            Some(due) if due > now => due.duration_since(now).as_millis().min(1000) as i32 + 1,
            Some(_) => 0,
            None => 1000,
        };
        let mut fds = [
            ffi::PollFd { fd: wayland_fd, events: ffi::POLLIN, revents: 0 },
            ffi::PollFd { fd: if stdin_open { stdin } else { -1 }, events: ffi::POLLIN, revents: 0 },
        ];
        let ready = unsafe { ffi::poll(fds.as_mut_ptr(), 2, timeout) };
        let wayland_readable = ready > 0 && fds[0].revents & (ffi::POLLIN | ffi::POLLERR | ffi::POLLHUP) != 0;
        app.wayland.finish_read(wayland_readable)?;

        if stdin_open && ready > 0 && fds[1].revents != 0 {
            let mut buffer = [0u8; 65536];
            loop {
                let count = unsafe { ffi::read(stdin, buffer.as_mut_ptr().cast(), buffer.len()) };
                if count > 0 {
                    input.extend_from_slice(&buffer[..count as usize]);
                } else if count == 0 {
                    stdin_open = false;
                    app.quit = true;
                    break;
                } else {
                    break;
                }
            }
            while let Some(end) = input.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = input.drain(..=end).collect();
                let line = String::from_utf8_lossy(&line);
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                match protocol::parse(line) {
                    Ok(command) => app.apply(command),
                    Err(error) => log!("{error}"),
                }
            }
        }

        for screen in app.screens.iter_mut() {
            screen.handle_configure();
            if screen.window.closed() && standalone {
                app.quit = true;
            }
        }
        app.tick();
        app.wayland.flush();
    }
    log!("koniec");
    let screens = std::mem::take(&mut app.screens);
    for screen in screens {
        screen.destroy(&app.wayland, &app.egl);
    }
    app.wayland.flush();
    Ok(())
}

impl App {
    fn power(&self) -> Power {
        if self.saver { Power::Saver } else { self.system_power }
    }

    fn apply(&mut self, command: Command) {
        let now = Instant::now();
        let cursor_only = matches!(command, Command::Cursor(_));
        match command {
            Command::Monitors(monitors) => self.sync_screens(monitors),
            Command::Windows { moving, list } => {
                if moving && !self.desktop.moving {
                    self.desktop.started_moving = true;
                }
                self.desktop.moving = moving;
                if list != self.desktop.windows {
                    self.desktop.windows = list;
                    self.desktop.generation += 1;
                }
            }
            Command::Elsewhere(ids) => self.desktop.elsewhere = ids.into_iter().collect(),
            Command::Workspace => {
                self.screens.iter_mut().for_each(|s| s.simulation.space_changed());
            }
            Command::Slide(velocity) => {
                self.screens.iter_mut().for_each(|s| s.simulation.space_slid(velocity));
            }
            Command::Cursor(position) => {
                self.desktop.cursor = position;
                if let Some((x, y)) = position {
                    for screen in self.screens.iter_mut() {
                        let p = screen.local(x, y);
                        if p.x >= 0.0 && p.x <= screen.simulation.width() && p.y >= -20.0 && p.y <= screen.simulation.height() {
                            screen.wake(now + Duration::from_millis(4));
                        }
                    }
                }
            }
            Command::Covered { all, fullscreen } => {
                self.desktop.all_covered = all;
                self.desktop.fullscreen = fullscreen.into_iter().collect::<HashSet<_>>();
            }
            Command::Snowfall(snowfall) => {
                self.settings.snowfall = snowfall;
                self.screens.iter_mut().for_each(|s| s.simulation.set_snowfall(snowfall));
            }
            Command::Accumulate(on) => {
                self.settings.accumulate = on;
                self.screens.iter_mut().for_each(|s| s.simulation.accumulates = on);
            }
            Command::Melt(on) => {
                self.settings.melt = on;
                self.screens.iter_mut().for_each(|s| s.simulation.thaws = on);
            }
            Command::SnowOnWindows(on) => {
                self.settings.snow_on_windows = on;
                if !on {
                    self.screens.iter_mut().for_each(Screen::drop_window_snow);
                }
            }
            Command::OverWindows(on) => self.settings.over_windows = on,
            Command::CalmFps(fps) => self.settings.calm_fps = fps,
            Command::Sharp(on) => {
                self.settings.sharp = on;
                self.screens.iter_mut().for_each(|s| s.apply_sharpness(on));
            }
            Command::Power(power) => {
                self.saver = power == Power::Saver;
                self.apply_density();
            }
            Command::ClearSnow(calm) => {
                let direction = self.screens.first().map_or(-1.0, |s| s.simulation.breeze_direction());
                self.screens.iter_mut().for_each(|s| s.simulation.blow_away(direction, calm));
            }
            Command::Halloween(on) => {
                self.settings.halloween = on;
                self.update_halloween();
            }
            Command::Pause(on) => self.desktop.paused = on,
            Command::Quit => self.quit = true,
        }
        for screen in &mut self.screens {
            if !(cursor_only && screen.simulation.is_empty()) {
                screen.wake(now);
            }
        }
    }

    fn update_halloween(&mut self) {
        let wanted = self.season && self.settings.halloween;
        if wanted != self.halloween {
            self.halloween = wanted;
            log!("Halloween: {}", if wanted { "tak" } else { "nie" });
            self.screens.iter_mut().for_each(|s| s.simulation.set_halloween(wanted));
        }
        self.night = halloween::night(wanted);
    }

    fn apply_density(&mut self) {
        let density = if self.power() == Power::Saver { 0.75 } else { 1.0 };
        self.screens.iter_mut().for_each(|s| s.simulation.set_density(density));
    }

    fn sync_screens(&mut self, monitors: Vec<Monitor>) {
        let now = Instant::now();
        let mut kept = Vec::new();
        let mut current = std::mem::take(&mut self.screens);
        let all = monitors.clone();
        for monitor in monitors {
            if let Some(i) = current.iter().position(|s| s.monitor.index == monitor.index) {
                let mut screen = current.swap_remove(i);
                screen.update_monitor(monitor, &self.settings);
                kept.push(screen);
                continue;
            }
            match Screen::new(&self.wayland, &self.egl, monitor, &self.settings, now) {
                Ok(screen) => kept.push(screen),
                Err(error) => log!("nie można utworzyć nakładki: {error}"),
            }
        }
        for screen in current {
            log!("monitor {} odłączony", screen.monitor.index);
            screen.destroy(&self.wayland, &self.egl);
        }
        kept.sort_by_key(|s| s.monitor.index);
        for screen in &mut kept {
            screen.simulation.walls = desktop::walls(&screen.monitor, &all);
        }
        for screen in &mut kept {
            screen.simulation.set_halloween(self.halloween);
        }
        self.screens = kept;
        self.apply_density();
    }

    fn tick(&mut self) {
        let now = Instant::now();
        if now >= self.chores_at {
            self.chores_at = now + Duration::from_secs(2);
            let power = light::power_from_system();
            if power != self.system_power {
                self.system_power = power;
                log!("zasilanie: {power:?}");
                self.apply_density();
            }
            let season = light::is_halloween();
            if season != self.season {
                self.season = season;
                self.update_halloween();
            }
            if now >= self.light_at {
                self.light_at = now + Duration::from_secs(60);
                self.light = light::daylight();
            }
        }
        if self.desktop.take_started_moving() && self.settings.snow_on_windows {
            self.screens.iter_mut().for_each(|s| s.wake(now));
        }
        let time = (now.duration_since(self.started).as_secs_f64() % 1000.0) as f32;
        let tolerance = Duration::from_millis(2);
        let power = self.power();
        for screen in self.screens.iter_mut().filter(|s| s.next_due <= now + tolerance) {
            let began = Instant::now();
            match screen.frame(&mut self.renderer, &self.egl, &self.desktop, &self.settings, power, (self.light, self.night), time, now) {
                Ok(drew) => self.stats.frame(drew, began.elapsed(), screen.simulation.sprite_count(), screen.damaged),
                Err(error) => {
                    log!("klatka nieudana: {error}");
                    screen.next_due = now + Duration::from_secs(1);
                }
            }
        }
        self.stats.report(now);
    }
}

struct Stats {
    since: Instant,
    steps: u32,
    draws: u32,
    busy: Duration,
    sprites: usize,
    damaged: f64,
}

impl Stats {
    fn new(now: Instant) -> Stats {
        Stats { since: now, steps: 0, draws: 0, busy: Duration::ZERO, sprites: 0, damaged: 0.0 }
    }

    fn frame(&mut self, drew: bool, busy: Duration, sprites: usize, damaged: f32) {
        self.steps += 1;
        self.draws += drew as u32;
        if drew {
            self.damaged += damaged as f64;
        }
        self.busy += busy;
        self.sprites = self.sprites.max(sprites);
    }

    fn report(&mut self, now: Instant) {
        let elapsed = now.duration_since(self.since).as_secs_f64();
        if elapsed < 600.0 {
            return;
        }
        let busy = self.busy.as_secs_f64();
        log!(
            "klatki: {:.1} kroków/s, {:.1} rysowań/s, {:.0} µs na krok, {:.2}% rdzenia, do {} płatków, przerysowane średnio {:.0}% ekranu",
            self.steps as f64 / elapsed,
            self.draws as f64 / elapsed,
            busy * 1e6 / self.steps.max(1) as f64,
            busy / elapsed * 100.0,
            self.sprites,
            self.damaged * 100.0 / self.draws.max(1) as f64
        );
        *self = Stats::new(now);
    }
}

fn snapshot(path: &str, seconds: f32) -> Result<(), String> {
    use snow_core::geometry::{Rect, V2};
    use snow_core::simulation::{SnowSimulation, Snowfall, WindowSurface};

    let (width, height) = (960, 600);
    let egl = Egl::new(Platform::Surfaceless)?;
    egl.make_current(std::ptr::null_mut())?;
    let gl = gl::Gl::load(&gl::lookup_with(|name| egl.get_proc(name), ffi::Library::open(&["libGLESv2.so.2", "libGLESv2.so"]).ok()))?;
    let mut renderer = Renderer::new(gl)?;
    let g = &renderer.gl;
    unsafe {
        let (mut framebuffer, mut color) = (0, 0);
        (g.GenFramebuffers)(1, &mut framebuffer);
        (g.BindFramebuffer)(gl::FRAMEBUFFER, framebuffer);
        (g.GenRenderbuffers)(1, &mut color);
        (g.BindRenderbuffer)(gl::RENDERBUFFER, color);
        (g.RenderbufferStorage)(gl::RENDERBUFFER, gl::RGBA8, width, height);
        (g.FramebufferRenderbuffer)(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, color);
        if (g.CheckFramebufferStatus)(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            return Err("bufor testowy niekompletny".into());
        }
    }

    let mut simulation = SnowSimulation::new(width as f32, height as f32, Snowfall::Heavy);
    simulation.thaws = false;
    let spooky = halloween::previewed();
    simulation.set_halloween(spooky);
    let mut desktop = Desktop::new();
    desktop.windows = vec![
        desktop::Window { id: 1, x: 520, y: 200, width: 360, height: 240, flags: desktop::FLAG_ROUNDED },
        desktop::Window { id: 2, x: 120, y: 120, width: 460, height: 300, flags: 0 },
    ];
    let area = Area { x: 0, y: 0, width, height };
    let surfaces: Vec<WindowSurface> = desktop.surfaces(&area, (width as f32, height as f32));
    simulation.update_windows(&surfaces, 0.2, &HashSet::new(), &HashSet::new(), false);
    let dt = 1.0 / 30.0;
    for _ in 0..(seconds / dt) as usize {
        simulation.step(dt, screen::AWAY);
    }
    for i in 0..60 {
        simulation.step(dt, V2::new(300.0 + i as f32 * 6.0, 6.0));
    }
    if spooky {
        for _ in 0..(20.0 / dt) as usize {
            simulation.step(dt, screen::AWAY);
        }
    }
    for _ in 0..10 {
        simulation.step(dt, screen::AWAY);
    }
    let ambient = render::Ambient {
        light: light::daylight_at(12.0),
        shadow_tint: [0.0; 4],
        night: halloween::night(spooky),
        lantern: V2::new(width as f32 * 0.75, 40.0),
    };
    let masks = if std::env::var_os("SNOWHUSH_BEHIND").is_some() { desktop.masks(&area, (width as f32, height as f32)) } else { Vec::new() };
    renderer.draw(&simulation, (width, height), 1.0, ambient, 3.0, &masks)?;
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    unsafe {
        (renderer.gl.PixelStorei)(gl::PACK_ALIGNMENT, 1);
        (renderer.gl.ReadPixels)(0, 0, width, height, gl::RGBA, gl::UNSIGNED_BYTE, pixels.as_mut_ptr().cast());
    }

    let mut out = Vec::with_capacity((width * height * 3) as usize + 32);
    out.extend_from_slice(format!("P6\n{width} {height}\n255\n").as_bytes());
    let windows: Vec<Rect> = surfaces.iter().map(|s| s.frame).collect();
    for row in (0..height).rev() {
        for column in 0..width {
            let i = ((row * width + column) * 4) as usize;
            let (x, y) = (column as f32 + 0.5, row as f32 + 0.5);
            let inside = windows.iter().position(|r| x >= r.x && x < r.max_x() && y >= r.y && y < r.max_y());
            let background: [f32; 3] = match inside {
                Some(0) => [0.20, 0.24, 0.30],
                Some(_) => [0.26, 0.22, 0.30],
                None => [0.08 + 0.05 * (y / height as f32), 0.10 + 0.06 * (y / height as f32), 0.16 + 0.08 * (y / height as f32)],
            };
            let a = pixels[i + 3] as f32 / 255.0;
            for c in 0..3 {
                let value = pixels[i + c] as f32 / 255.0 + background[c] * (1.0 - a);
                out.push((value.clamp(0.0, 1.0) * 255.0) as u8);
            }
        }
    }
    std::fs::write(path, out).map_err(|e| e.to_string())?;
    log!("zapisano {path}: {} płatków, {} warstw zasp", simulation.sprite_count(), surfaces.len() + 1);
    Ok(())
}
