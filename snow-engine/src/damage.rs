use std::collections::HashMap;

use snow_core::simulation::{COLUMN_WIDTH, Sprite};

const TILE: i32 = 16;
const GLINT_EVERY: u32 = 6;
const MAX_RECTS: usize = 60;

pub struct PileShape {
    pub origin: (f32, f32),
    pub count: usize,
    pub bottom: f32,
    pub top: f32,
    pub heights: Vec<f32>,
}

pub struct Damage {
    size: (i32, i32),
    columns: i32,
    rows: i32,
    previous: Vec<bool>,
    current: Vec<bool>,
    piles: HashMap<(u32, u32, usize), Vec<f32>>,
    frames: u32,
    full: bool,
    pub last_fraction: f32,
}

impl Damage {
    pub fn new() -> Damage {
        Damage { size: (0, 0), columns: 0, rows: 0, previous: Vec::new(), current: Vec::new(), piles: HashMap::new(), frames: 0, full: true, last_fraction: 1.0 }
    }

    pub fn invalidate(&mut self) {
        self.full = true;
    }

    pub fn compute(&mut self, size: (i32, i32), scale: f32, sprites: &[Sprite], piles: &[PileShape]) -> Option<Vec<i32>> {
        if size != self.size {
            self.size = size;
            self.columns = (size.0 + TILE - 1) / TILE;
            self.rows = (size.1 + TILE - 1) / TILE;
            let tiles = (self.columns * self.rows).max(0) as usize;
            self.previous = vec![false; tiles];
            self.current = vec![false; tiles];
            self.full = true;
        }
        self.current.iter_mut().for_each(|tile| *tile = false);
        self.frames = self.frames.wrapping_add(1);

        for sprite in sprites {
            let (half, x, y) = sprite_bounds(sprite, scale);
            self.mark(x - half, y - half, x + half, y + half);
        }

        let glints = self.frames.is_multiple_of(GLINT_EVERY);
        let mut seen = HashMap::with_capacity(piles.len());
        for pile in piles {
            let key = (pile.origin.0.to_bits(), pile.origin.1.to_bits(), pile.count);
            let changed = self.piles.get(&key).is_none_or(|old| old != &pile.heights);
            if changed || glints {
                let x0 = pile.origin.0 * scale;
                let x1 = (pile.origin.0 + pile.count as f32 * COLUMN_WIDTH) * scale;
                let y0 = (pile.origin.1 + pile.bottom) * scale;
                let y1 = (pile.origin.1 + pile.top + 2.0) * scale;
                self.mark(x0, y0, x1, y1);
            }
            seen.insert(key, pile.heights.clone());
        }
        self.piles = seen;

        let full = std::mem::take(&mut self.full);
        let dirty: Vec<bool> = self.current.iter().zip(&self.previous).map(|(a, b)| *a || *b).collect();
        std::mem::swap(&mut self.previous, &mut self.current);
        if full {
            self.last_fraction = 1.0;
            return None;
        }
        for factor in [1, 4, 8, 16] {
            let rects = self.runs(&dirty, factor);
            if rects.len() / 4 <= MAX_RECTS {
                let area: i64 = rects.chunks(4).map(|r| r[2] as i64 * r[3] as i64).sum();
                self.last_fraction = area as f32 / (self.size.0 as f32 * self.size.1 as f32).max(1.0);
                if self.last_fraction > 0.85 {
                    break;
                }
                return Some(rects);
            }
        }
        self.last_fraction = 1.0;
        None
    }

    fn runs(&self, dirty: &[bool], factor: i32) -> Vec<i32> {
        let columns = (self.columns + factor - 1) / factor;
        let rows = (self.rows + factor - 1) / factor;
        let cell = TILE * factor;
        let marked = |column: i32, row: i32| {
            (row * factor..((row + 1) * factor).min(self.rows))
                .any(|r| (column * factor..((column + 1) * factor).min(self.columns)).any(|c| dirty[(r * self.columns + c) as usize]))
        };
        let mut rects = Vec::new();
        for row in 0..rows {
            let mut column = 0;
            while column < columns {
                if !marked(column, row) {
                    column += 1;
                    continue;
                }
                let start = column;
                while column < columns && marked(column, row) {
                    column += 1;
                }
                let x = start * cell;
                let y = row * cell;
                rects.extend_from_slice(&[x, y, ((column * cell).min(self.size.0) - x).max(1), ((y + cell).min(self.size.1) - y).max(1)]);
            }
        }
        rects
    }

    fn mark(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        if self.columns == 0 || self.rows == 0 {
            return;
        }
        let c0 = ((x0.floor() as i32) / TILE).clamp(0, self.columns - 1);
        let c1 = ((x1.ceil() as i32) / TILE).clamp(0, self.columns - 1);
        let r0 = ((y0.floor() as i32) / TILE).clamp(0, self.rows - 1);
        let r1 = ((y1.ceil() as i32) / TILE).clamp(0, self.rows - 1);
        if x1 < 0.0 || y1 < 0.0 || x0 > self.size.0 as f32 || y0 > self.size.1 as f32 {
            return;
        }
        for row in r0..=r1 {
            let base = (row * self.columns) as usize;
            for column in c0..=c1 {
                self.current[base + column as usize] = true;
            }
        }
    }
}

fn sprite_bounds(sprite: &Sprite, scale: f32) -> (f32, f32, f32) {
    let pixels = sprite.size * scale;
    let travel = (sprite.velocity.x * 0.018, sprite.velocity.y * 0.018);
    let travel_length = (travel.0 * travel.0 + travel.1 * travel.1).sqrt();
    let smear = travel_length.min(sprite.size * if sprite.lump > 0.0 { 1.2 } else { 2.2 });
    let streak = if smear > 0.25 { smear * scale } else { 0.0 };
    let reach = if sprite.lump > 0.0 { 1.3 } else { 1.0 };
    let extent = (pixels * reach + streak).max(2.0);
    (extent * 0.5 + 2.0, sprite.position.x * scale, sprite.position.y * scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use snow_core::geometry::V2;

    fn flake(x: f32, y: f32) -> Sprite {
        Sprite { position: V2::new(x, y), size: 3.0, alpha: 1.0, ..Default::default() }
    }

    #[test]
    fn reports_only_where_snow_moved() {
        let mut damage = Damage::new();
        let size = (1920, 1080);
        assert!(damage.compute(size, 1.0, &[flake(100.0, 100.0)], &[]).is_none(), "pierwsza klatka – cały ekran");
        let rects = damage.compute(size, 1.0, &[flake(101.0, 99.0)], &[]).unwrap();
        let area: i32 = rects.chunks(4).map(|r| r[2] * r[3]).sum();
        assert!(area > 0 && area < 4 * 16 * 16 * 2, "mały obszar wokół płatka: {area}");
        assert!(!damage.compute(size, 1.0, &[], &[]).unwrap().is_empty());
        assert!(damage.compute(size, 1.0, &[], &[]).unwrap().is_empty());
    }

    #[test]
    fn unchanged_piles_are_quiet_between_glints() {
        let mut damage = Damage::new();
        let size = (800, 600);
        let pile = || PileShape { origin: (0.0, 0.0), count: 400, bottom: -5.0, top: 20.0, heights: vec![1.0; 800] };
        damage.compute(size, 1.0, &[], &[pile()]);
        let mut quiet = 0;
        for _ in 0..6 {
            if damage.compute(size, 1.0, &[], &[pile()]).unwrap().is_empty() {
                quiet += 1;
            }
        }
        assert!(quiet >= 2, "między odświeżeniami iskierek nieruchoma zaspa nic nie zgłasza ({quiet})");
    }
}

#[cfg(test)]
mod measure {
    use super::*;
    use snow_core::simulation::{SnowSimulation, Snowfall};

    #[test]
    #[ignore]
    fn coverage_on_a_laptop_screen() {
        for snowfall in [Snowfall::Light, Snowfall::Steady, Snowfall::Heavy] {
            let mut simulation = SnowSimulation::new(1920.0, 1171.0, snowfall);
            let mut damage = Damage::new();
            let mut buffer = vec![Sprite::default(); 20000];
            let mut heights = vec![0.0; 8192];
            let (mut sum, mut frames) = (0.0, 0);
            for frame in 0..30 * 90 {
                simulation.step(1.0 / 30.0, snow_core::geometry::V2::new(-1e4, -1e4));
                let n = simulation.write_sprites(&mut buffer);
                let piles: Vec<PileShape> = simulation
                    .write_layers(&mut heights)
                    .iter()
                    .map(|p| PileShape { origin: (p.origin.x, p.origin.y), count: p.count, bottom: p.bottom, top: p.top, heights: heights[p.offset..p.offset + 2 * p.count].to_vec() })
                    .collect();
                damage.compute((1920, 1171), 1.0, &buffer[..n], &piles);
                if frame > 30 * 30 {
                    sum += damage.last_fraction;
                    frames += 1;
                }
            }
            let n = simulation.write_sprites(&mut buffer);
            let rects = damage.compute((1920, 1171), 1.0, &buffer[..n], &[]).map_or(0, |r| r.len() / 4);
            println!("{snowfall:?}: przerysowane średnio {:.1}% ekranu, {} płatków, prostokątów {rects}", sum * 100.0 / frames as f32, simulation.sprite_count());
        }
    }
}
