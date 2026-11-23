//! Reprezentacja pojedynczej piłki oraz jej efektu śladu (ring buffer).

use macroquad::prelude::*;
use macroquad::rand::gen_range;

/// Liczba zapamiętanych pozycji w śladzie za piłką.
pub const TRAIL_LEN: usize = 22;

/// Gęstość powierzchniowa - masa = gęstość * π * r².
const DENSITY: f32 = 1.0;

/// Ring buffer ostatnich pozycji. Stały rozmiar, zero alokacji w trakcie symulacji.
#[derive(Clone, Debug)]
pub struct Trail {
    points: [Vec2; TRAIL_LEN],
    head: usize,
    count: usize,
}

impl Trail {
    pub fn new() -> Self {
        Self {
            points: [Vec2::ZERO; TRAIL_LEN],
            head: 0,
            count: 0,
        }
    }

    /// Skalowanie śladu przy zmianie rozmiaru okna.
    pub fn scale(&mut self, sx: f32, sy: f32) {
        for p in self.points.iter_mut() {
            p.x *= sx;
            p.y *= sy;
        }
    }

    pub fn push(&mut self, point: Vec2) {
        self.points[self.head] = point;
        self.head = (self.head + 1) % TRAIL_LEN;
        self.count = (self.count + 1).min(TRAIL_LEN);
    }

    /// Iteracja od najstarszego punktu do najnowszego.
    pub fn iter(&self) -> impl Iterator<Item = Vec2> + '_ {
        let start = (self.head + TRAIL_LEN - self.count) % TRAIL_LEN;
        (0..self.count).map(move |i| self.points[(start + i) % TRAIL_LEN])
    }
}

impl Default for Trail {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct Ball {
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f32,
    pub mass: f32,
    pub color: Color,
    pub trail: Trail,
    /// Piłka chwycona prawym przyciskiem myszy traktowana jest jako nieruchoma
    /// (nieskończona masa) dla celów kolizji.
    pub held: bool,
}

impl Ball {
    pub fn new(pos: Vec2, vel: Vec2, radius: f32, color: Color) -> Self {
        Self {
            pos,
            vel,
            radius,
            mass: DENSITY * std::f32::consts::PI * radius * radius,
            color,
            trail: Trail::new(),
            held: false,
        }
    }

    #[inline]
    pub fn inv_mass(&self) -> f32 {
        if self.held {
            0.0
        } else {
            1.0 / self.mass
        }
    }

    pub fn set_held(&mut self, held: bool) {
        self.held = held;
        if held {
            self.vel = Vec2::ZERO;
        }
    }

    #[inline]
    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.vel.length_squared()
    }

    #[inline]
    pub fn contains(&self, point: Vec2) -> bool {
        (point - self.pos).length_squared() <= self.radius * self.radius
    }

    pub fn draw(&self, show_velocity: bool) {
        // Zanikający ślad.
        let n = self.trail.count;
        if n > 1 {
            for (i, p) in self.trail.iter().enumerate() {
                let t = (i + 1) as f32 / n as f32;
                let alpha = 0.30 * t * t;
                let r = self.radius * (0.20 + 0.65 * t);
                draw_circle(p.x, p.y, r, self.color.with_alpha(alpha));
            }
        }

        draw_circle(self.pos.x, self.pos.y, self.radius, self.color);
        draw_circle_lines(
            self.pos.x,
            self.pos.y,
            self.radius,
            1.5,
            Color::new(0.0, 0.0, 0.0, 0.35),
        );

        // Mały refleks świetlny nadaje objętość.
        draw_circle(
            self.pos.x - self.radius * 0.30,
            self.pos.y - self.radius * 0.30,
            self.radius * 0.26,
            Color::new(1.0, 1.0, 1.0, 0.28),
        );

        if self.held {
            draw_circle_lines(
                self.pos.x,
                self.pos.y,
                self.radius + 5.0,
                2.0,
                Color::new(1.0, 1.0, 1.0, 0.85),
            );
        }

        if show_velocity {
            draw_velocity_vector(self.pos, self.vel);
        }
    }
}

fn draw_velocity_vector(pos: Vec2, vel: Vec2) {
    let speed = vel.length();
    if speed < 1.0 {
        return;
    }
    let scale = 0.09_f32.min(70.0 / speed);
    let tip = pos + vel * scale;
    let color = Color::new(1.0, 0.92, 0.35, 0.85);
    draw_line(pos.x, pos.y, tip.x, tip.y, 2.0, color);

    let dir = vel / speed;
    let perp = Vec2::new(-dir.y, dir.x);
    let back = tip - dir * 9.0;
    draw_line(
        tip.x,
        tip.y,
        back.x + perp.x * 5.0,
        back.y + perp.y * 5.0,
        2.0,
        color,
    );
    draw_line(
        tip.x,
        tip.y,
        back.x - perp.x * 5.0,
        back.y - perp.y * 5.0,
        2.0,
        color,
    );
}

/// Losowa piłka wokół zadanego punktu, z losowym promieniem i kolorem HSL.
pub fn random_ball(pos: Vec2, bounds: Vec2, velocity: Vec2) -> Ball {
    let radius = gen_range(9.0_f32, 27.0_f32);
    let margin = radius + 1.0;
    let px = pos.x.clamp(margin, (bounds.x - margin).max(margin));
    let py = pos.y.clamp(margin, (bounds.y - margin).max(margin));
    let color = hsv_to_rgb(
        gen_range(0.0_f32, 1.0_f32),
        gen_range(0.55_f32, 0.85_f32),
        0.95,
    );
    Ball::new(vec2(px, py), velocity, radius, color)
}

/// Konwersja HSV (h, s, v w <0,1>) na kolor macroquad.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let h = h.rem_euclid(1.0) * 6.0;
    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h.floor() as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color::new(r + m, g + m, b + m, 1.0)
}
