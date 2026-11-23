//! Prosta animacja eksplozji: rozszerzające się pierścienie, błysk i promienie.

use macroquad::prelude::*;

const DURATION: f32 = 0.7;
const RAYS: usize = 18;

pub struct ExplosionFx {
    center: Vec2,
    power: f32,
    age: f32,
}

impl ExplosionFx {
    pub fn new(center: Vec2, power: f32) -> Self {
        Self {
            center,
            power,
            age: 0.0,
        }
    }

    /// Animacja liczona w czasie rzeczywistym (niezależnie od skali czasu fizyki).
    pub fn update(&mut self, dt: f32) {
        self.age += dt;
    }

    pub fn finished(&self) -> bool {
        self.age >= DURATION
    }

    pub fn draw(&self) {
        let t = (self.age / DURATION).clamp(0.0, 1.0);
        let ease = 1.0 - (1.0 - t).powi(3);
        let max_radius = 70.0 + self.power * 0.20;

        // Błysk w środku - szybko zanika.
        let flash = (1.0 - t).powi(2);
        if flash > 0.01 {
            let flash_radius = 30.0 + 55.0 * flash;
            draw_circle(
                self.center.x,
                self.center.y,
                flash_radius,
                Color::new(1.0, 0.96, 0.72, 0.55 * flash),
            );
            draw_circle(
                self.center.x,
                self.center.y,
                flash_radius * 0.55,
                Color::new(1.0, 1.0, 0.95, 0.7 * flash),
            );
        }

        // Promienie radialne.
        let ray_alpha = 0.75 * (1.0 - t).powi(2);
        if ray_alpha > 0.01 {
            for i in 0..RAYS {
                let angle = (i as f32 / RAYS as f32) * std::f32::consts::TAU;
                let direction = vec2(angle.cos(), angle.sin());
                let inner = max_radius * 0.08 * ease;
                let outer = max_radius * (0.35 + 0.65 * ease);
                let a = self.center + direction * inner;
                let b = self.center + direction * outer;
                let color = if i % 2 == 0 {
                    Color::new(1.0, 0.82, 0.35, ray_alpha)
                } else {
                    Color::new(1.0, 0.55, 0.25, ray_alpha * 0.7)
                };
                draw_line(a.x, a.y, b.x, b.y, 2.5 * (1.0 - t) + 0.5, color);
            }
        }

        // Trzy pierścienie z lekkim opóźnieniem.
        for ring in 0..3 {
            let delay = ring as f32 * 0.07;
            if self.age < delay {
                continue;
            }
            let rt = ((self.age - delay) / (DURATION - delay)).clamp(0.0, 1.0);
            if rt >= 1.0 {
                continue;
            }
            let re = 1.0 - (1.0 - rt).powi(3);
            let radius = max_radius * (0.30 + 0.70 * re) * (1.0 - ring as f32 * 0.16);
            let alpha = (1.0 - rt) * (0.85 - ring as f32 * 0.22);
            let hue = 1.0 - ring as f32 * 0.18;
            draw_circle_lines(
                self.center.x,
                self.center.y,
                radius,
                5.0 * (1.0 - rt) + 1.0,
                Color::new(1.0, 0.55 + hue * 0.30, 0.25 + hue * 0.25, alpha),
            );
        }
    }
}
