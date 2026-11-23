//! Własny silnik fizyki 2D: całkowanie ze stałym krokiem, grawitacja, opór,
//! tarcie, kolizje ze ścianami oraz sprężyste zderzenia piłek z korekcją pozycji.

use std::collections::HashMap;

use macroquad::prelude::*;
use macroquad::rand::gen_range;

use crate::ball::{random_ball, Ball};

/// Stały krok fizyki - 120 Hz. Stabilność nie zależy od liczby klatek na sekundę.
pub const FIXED_DT: f32 = 1.0 / 120.0;

const SOLVER_ITERATIONS: usize = 3;
const MAX_SUBSTEPS: usize = 16;
const MAX_SPEED: f32 = 4000.0;
const POSITION_SLOP: f32 = 0.01;
const POSITION_PERCENT: f32 = 0.80;

#[derive(Clone, Debug)]
pub struct PhysicsConfig {
    /// Przyspieszenie grawitacyjne w pikselach/s².
    pub gravity: f32,
    /// Współczynnik restytucji (0 = całkowicie nieelastycznie, 1 = idealnie sprzężyście).
    pub restitution: f32,
    /// Liniowy współczynnik oporu powietrza [1/s].
    pub air_drag: f32,
    /// Tarcie toczące się po dolnej krawędzi [1/s].
    pub floor_friction: f32,
    /// Wymiary planszy w pikselach.
    pub bounds: Vec2,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: 900.0,
            restitution: 0.86,
            air_drag: 0.10,
            floor_friction: 1.6,
            bounds: vec2(1280.0, 800.0),
        }
    }
}

pub struct World {
    pub balls: Vec<Ball>,
    pub config: PhysicsConfig,
    /// 0.1 - 2.0 (slow motion / przyspieszenie).
    pub time_scale: f32,
    accumulator: f32,
    /// Siatka przestrzenna do szybkiej detekcji sąsiedztw (zamiast pełnego O(n²)).
    grid: HashMap<(i32, i32), Vec<usize>>,
}

impl World {
    pub fn new(bounds: Vec2) -> Self {
        let mut config = PhysicsConfig::default();
        config.bounds = bounds;
        Self {
            balls: Vec::new(),
            config,
            time_scale: 1.0,
            accumulator: 0.0,
            grid: HashMap::with_capacity(512),
        }
    }

    /// Akumulator czasu zerowany przy pauzie / resecie - zapobiega "skokowi" fizyki.
    pub fn flush(&mut self) {
        self.accumulator = 0.0;
    }

    pub fn clear(&mut self) {
        self.balls.clear();
        self.flush();
    }

    pub fn reset(&mut self) {
        self.clear();
        let b = self.config.bounds;
        for _ in 0..14 {
            let r = gen_range(10.0_f32, 26.0_f32);
            let x = r + gen_range(0.0_f32, (b.x - 2.0 * r).max(0.0));
            let y = r + gen_range(0.0_f32, (b.y * 0.5 - 2.0 * r).max(0.0));
            let vel = vec2(
                gen_range(-260.0_f32, 260.0_f32),
                gen_range(-60.0_f32, 60.0_f32),
            );
            self.balls.push(random_ball(vec2(x, y), b, vel));
        }
    }

    pub fn add_random(&mut self, count: usize) {
        let b = self.config.bounds;
        for _ in 0..count {
            let mut chosen = None;
            // Kilka prób odrzucenia, żeby nowe piłki nie startowały wewnątrz innych.
            for _ in 0..12 {
                let r = gen_range(8.0_f32, 26.0_f32);
                let x = r + gen_range(0.0_f32, (b.x - 2.0 * r).max(0.0));
                let y = r + gen_range(0.0_f32, (b.y - 2.0 * r).max(0.0));
                let p = vec2(x, y);
                let free = self.balls.iter().all(|other| {
                    (other.pos - p).length_squared()
                        > (other.radius + r + 2.0) * (other.radius + r + 2.0)
                });
                chosen = Some((p, r));
                if free {
                    break;
                }
            }
            if let Some((p, r)) = chosen {
                let vel = vec2(
                    gen_range(-240.0_f32, 240.0_f32),
                    gen_range(-120.0_f32, 40.0_f32),
                );
                let color = crate::ball::hsv_to_rgb(
                    gen_range(0.0_f32, 1.0_f32),
                    gen_range(0.55_f32, 0.85_f32),
                    0.95,
                );
                let ball = Ball::new(p, vel, r, color);
                self.balls.push(ball);
            }
        }
    }

    /// Znajduje piłkę pod kursorem (od najbardziej wierzchniej).
    pub fn pick_ball(&self, point: Vec2) -> Option<usize> {
        self.balls.iter().rposition(|ball| ball.contains(point))
    }

    /// Impuls radialny od wybranego punktu o zadanej mocy.
    pub fn explode_at(&mut self, center: Vec2, power: f32) {
        for ball in &mut self.balls {
            if ball.held {
                continue;
            }
            let delta = ball.pos - center;
            let dir = if delta.length_squared() > 4.0 {
                delta.normalize()
            } else {
                Vec2::new(gen_range(-1.0_f32, 1.0_f32), gen_range(-1.0_f32, 1.0_f32))
                    .normalize_or_zero()
            };
            let falloff = 1.0 / (1.0 + delta.length() / 420.0);
            ball.vel += dir * (power * falloff);
        }
    }

    /// Skalowanie granic i pozycji przy zmianie rozmiaru okna.
    pub fn resize(&mut self, width: f32, height: f32) {
        let old = self.config.bounds;
        if old.x <= 0.0 || old.y <= 0.0 {
            self.config.bounds = vec2(width, height);
            return;
        }
        let sx = width / old.x;
        let sy = height / old.y;
        for ball in &mut self.balls {
            ball.pos.x =
                (ball.pos.x * sx).clamp(ball.radius, (width - ball.radius).max(ball.radius));
            ball.pos.y =
                (ball.pos.y * sy).clamp(ball.radius, (height - ball.radius).max(ball.radius));
            ball.trail.scale(sx, sy);
        }
        self.config.bounds = vec2(width, height);
    }

    /// Akumulator czasowy: tyle kroków stałej długości, ile "zebrało się" czasu.
    pub fn advance(&mut self, frame_time: f32) {
        let frame = frame_time.min(0.1); // limit po powrocie z minimizacji okna
        self.accumulator += frame * self.time_scale;
        let mut steps = 0;
        while self.accumulator >= FIXED_DT && steps < MAX_SUBSTEPS {
            self.step(FIXED_DT);
            self.accumulator -= FIXED_DT;
            steps += 1;
        }
        if steps == MAX_SUBSTEPS {
            self.accumulator = 0.0; // zapobieganie "spiral of death"
        }
    }

    fn step(&mut self, dt: f32) {
        let drag_factor = (-self.config.air_drag * dt).exp();
        let gravity = self.config.gravity;

        for ball in &mut self.balls {
            if ball.held {
                continue;
            }
            ball.vel.y += gravity * dt;
            ball.vel *= drag_factor;
            if ball.vel.length_squared() > MAX_SPEED * MAX_SPEED {
                ball.vel = ball.vel.normalize() * MAX_SPEED;
            }
            ball.pos += ball.vel * dt;
        }

        self.resolve_walls(dt);
        for _ in 0..SOLVER_ITERATIONS {
            self.resolve_ball_collisions();
            self.resolve_walls(dt);
        }
    }

    fn resolve_walls(&mut self, dt: f32) {
        let bounds = self.config.bounds;
        let e = self.config.restitution;
        let friction = (-self.config.floor_friction * dt).exp();

        for ball in &mut self.balls {
            if ball.held {
                continue;
            }
            let r = ball.radius;

            if ball.pos.x - r < 0.0 {
                ball.pos.x = r;
                if ball.vel.x < 0.0 {
                    ball.vel.x = -ball.vel.x * e;
                }
            } else if ball.pos.x + r > bounds.x {
                ball.pos.x = bounds.x - r;
                if ball.vel.x > 0.0 {
                    ball.vel.x = -ball.vel.x * e;
                }
            }

            if ball.pos.y - r < 0.0 {
                ball.pos.y = r;
                if ball.vel.y < 0.0 {
                    ball.vel.y = -ball.vel.y * e;
                }
            } else if ball.pos.y + r > bounds.y {
                ball.pos.y = bounds.y - r;
                if ball.vel.y > 0.0 {
                    ball.vel.y = -ball.vel.y * e;
                    // Małe odbicia wygaszamy, żeby piłka "uspokoiła się" na podłodze.
                    if ball.vel.y.abs() < 15.0 {
                        ball.vel.y = 0.0;
                    }
                }
                // Tarcie toczące się przy kontakcie z dolną krawędzią.
                ball.vel.x *= friction;
            }
        }
    }

    fn cell_key(&self, pos: Vec2, cell_size: f32) -> (i32, i32) {
        (
            (pos.x / cell_size).floor() as i32,
            (pos.y / cell_size).floor() as i32,
        )
    }

    /// Detekcja i rozwiązanie kolizji par piłek z wykorzystaniem siatki przestrzennej.
    fn resolve_ball_collisions(&mut self) {
        let n = self.balls.len();
        if n < 2 {
            return;
        }

        let max_radius = self.balls.iter().map(|b| b.radius).fold(0.0_f32, f32::max);
        let cell_size = (2.0 * max_radius).max(16.0);

        self.grid.clear();
        for (index, ball) in self.balls.iter().enumerate() {
            self.grid
                .entry(self.cell_key(ball.pos, cell_size))
                .or_default()
                .push(index);
        }

        let keys: Vec<(i32, i32)> = self
            .balls
            .iter()
            .map(|ball| self.cell_key(ball.pos, cell_size))
            .collect();

        for i in 0..n {
            let (cx, cy) = keys[i];
            for ox in -1..=1 {
                for oy in -1..=1 {
                    if let Some(candidates) = self.grid.get(&(cx + ox, cy + oy)) {
                        for &j in candidates {
                            if j > i {
                                let restitution = self.config.restitution;
                                resolve_pair(&mut self.balls, i, j, restitution);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn kinetic_energy(&self) -> f32 {
        self.balls.iter().map(Ball::kinetic_energy).sum()
    }
}

/// Sprężyste zderzenie 2D: impuls wzdłuż normalnej + korekcja penetracji.
/// Osobna funkcja (nie metoda), aby siatka przestrzenna mogła pozostać
/// niezmieniona podczas mutowania piłek.
fn resolve_pair(balls: &mut [Ball], i: usize, j: usize, restitution: f32) {
    let (left, right) = balls.split_at_mut(j);
    let a = &mut left[i];
    let b = &mut right[0];

    let delta = b.pos - a.pos;
    let dist_sq = delta.length_squared();
    let min_dist = a.radius + b.radius;
    if dist_sq >= min_dist * min_dist {
        return;
    }

    let dist = dist_sq.sqrt();
    let normal = if dist > 1.0e-5 {
        delta / dist
    } else {
        vec2(1.0, 0.0)
    };

    let inv_a = a.inv_mass();
    let inv_b = b.inv_mass();
    let inv_sum = inv_a + inv_b;
    if inv_sum <= 1.0e-9 {
        return;
    }

    // Pozycyjne rozdzielanie obiektów (proporcjonalnie do masy).
    let penetration = min_dist - dist;
    let correction =
        normal * (((penetration - POSITION_SLOP).max(0.0) * POSITION_PERCENT) / inv_sum);
    a.pos -= correction * inv_a;
    b.pos += correction * inv_b;

    // Impulsnormalny - zachowanie pędu przy masie ∝ πr².
    let relative_velocity = b.vel - a.vel;
    let velocity_along_normal = relative_velocity.dot(normal);
    if velocity_along_normal > 0.0 {
        return;
    }
    let impulse = -(1.0 + restitution) * velocity_along_normal / inv_sum;
    a.vel -= normal * (impulse * inv_a);
    b.vel += normal * (impulse * inv_b);

    // Niewielkie tarcie styczne zapobiega ślizganiu się piłek po sobie.
    let tangent = vec2(-normal.y, normal.x);
    let tangential_speed = (b.vel - a.vel).dot(tangent);
    let friction_impulse = (-tangential_speed / inv_sum) * 0.05;
    a.vel -= tangent * (friction_impulse * inv_a);
    b.vel += tangent * (friction_impulse * inv_b);
}
