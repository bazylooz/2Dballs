//! Lekki, własnoręcznie rysowany HUD (bez zewnętrznych zasobów i fontów).

use macroquad::prelude::*;

use crate::settings::AppSettings;

#[derive(Clone, Copy)]
struct UiRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl UiRect {
    const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    fn contains(self, p: Vec2) -> bool {
        p.x >= self.x && p.x <= self.x + self.w && p.y >= self.y && p.y <= self.y + self.h
    }
}

struct Slider {
    label: &'static str,
    min: f32,
    max: f32,
    value: f32,
    track: UiRect,
    decimals: usize,
    suffix: &'static str,
}

impl Slider {
    fn new(
        label: &'static str,
        min: f32,
        max: f32,
        value: f32,
        x: f32,
        y: f32,
        width: f32,
        decimals: usize,
        suffix: &'static str,
    ) -> Self {
        Self {
            label,
            min,
            max,
            value,
            // y to linia etykiety; suwak biegnie 14 px niżej.
            track: UiRect::new(x, y + 14.0, width, 10.0),
            decimals,
            suffix,
        }
    }

    fn hit_area(&self) -> UiRect {
        UiRect::new(
            self.track.x - 4.0,
            self.track.y - 20.0,
            self.track.w + 78.0,
            34.0,
        )
    }

    fn fraction(&self) -> f32 {
        (self.value - self.min) / (self.max - self.min)
    }

    fn set_from_mouse(&mut self, mouse: Vec2) {
        let t = ((mouse.x - self.track.x) / self.track.w).clamp(0.0, 1.0);
        self.value = self.min + t * (self.max - self.min);
    }

    fn value_text(&self) -> String {
        format!("{:.*}{}", self.decimals, self.value, self.suffix)
    }
}

struct Button {
    label: &'static str,
    rect: UiRect,
}

impl Button {
    const fn new(label: &'static str, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            label,
            rect: UiRect::new(x, y, w, h),
        }
    }
}

#[derive(Default)]
pub struct HudActions {
    pub clear: bool,
    pub add_20: bool,
    pub toggle_explosion: bool,
    pub toggle_vectors: bool,
    pub toggle_pause: bool,
    pub save_settings: bool,
    pub reset_settings: bool,
}

pub struct Stats {
    pub fps: f32,
    pub ball_count: usize,
    pub energy: f32,
    pub paused: bool,
    pub show_velocity: bool,
    pub explosion_armed: bool,
}

pub struct Hud {
    panel: UiRect,
    sliders: Vec<Slider>,
    buttons: Vec<Button>,
    dragging: Option<usize>,
    hover_slider: Option<usize>,
    hover_button: Option<usize>,
    status: Option<(String, f32)>,
}

impl Hud {
    pub fn new() -> Self {
        use crate::settings::{
            AIR_DRAG_RANGE, EXPLOSION_POWER_RANGE, GRAVITY_RANGE, RESTITUTION_RANGE,
            TIME_SCALE_RANGE,
        };

        let px = 12.0;
        let py = 12.0;
        let x = px + 14.0;
        let w = 200.0;

        let sliders = vec![
            Slider::new(
                "Grawitacja",
                GRAVITY_RANGE.0,
                GRAVITY_RANGE.1,
                900.0,
                x,
                py + 58.0,
                w,
                0,
                "",
            ),
            Slider::new(
                "Restytucja",
                RESTITUTION_RANGE.0,
                RESTITUTION_RANGE.1,
                0.86,
                x,
                py + 110.0,
                w,
                2,
                "",
            ),
            Slider::new(
                "Opor powietrza",
                AIR_DRAG_RANGE.0,
                AIR_DRAG_RANGE.1,
                0.10,
                x,
                py + 162.0,
                w,
                2,
                "",
            ),
            Slider::new(
                "Skala czasu",
                TIME_SCALE_RANGE.0,
                TIME_SCALE_RANGE.1,
                1.0,
                x,
                py + 214.0,
                w,
                2,
                "x",
            ),
            Slider::new(
                "Moc eksplozji",
                EXPLOSION_POWER_RANGE.0,
                EXPLOSION_POWER_RANGE.1,
                1700.0,
                x,
                py + 266.0,
                w,
                0,
                "",
            ),
        ];

        let bx = px + 14.0;
        let by = py + 320.0;
        let buttons = vec![
            Button::new("Wyczysc", bx, by, 86.0, 34.0),
            Button::new("+20 pilek", bx + 93.0, by, 90.0, 34.0),
            Button::new("Eksplozja", bx + 190.0, by, 86.0, 34.0),
            Button::new("Wektory", bx, by + 42.0, 132.0, 34.0),
            Button::new("Pauza", bx + 139.0, by + 42.0, 137.0, 34.0),
            Button::new("Zapisz", bx, by + 84.0, 132.0, 34.0),
            Button::new("Domyslne", bx + 139.0, by + 84.0, 137.0, 34.0),
        ];

        Self {
            panel: UiRect::new(px, py, 318.0, 548.0),
            sliders,
            buttons,
            dragging: None,
            hover_slider: None,
            hover_button: None,
            status: None,
        }
    }

    /// Wpisanie wartości z wczytanego pliku ustawień do suwaków.
    pub fn apply_settings(&mut self, settings: &AppSettings) {
        self.sliders[0].value = settings.gravity;
        self.sliders[1].value = settings.restitution;
        self.sliders[2].value = settings.air_drag;
        self.sliders[3].value = settings.time_scale;
        self.sliders[4].value = settings.explosion_power;
    }

    pub fn current_settings(&self, show_velocity: bool) -> AppSettings {
        AppSettings {
            gravity: self.gravity(),
            restitution: self.restitution(),
            air_drag: self.air_drag(),
            time_scale: self.time_scale(),
            explosion_power: self.explosion_power(),
            show_velocity,
        }
    }

    pub fn gravity(&self) -> f32 {
        self.sliders[0].value
    }

    pub fn restitution(&self) -> f32 {
        self.sliders[1].value
    }

    pub fn air_drag(&self) -> f32 {
        self.sliders[2].value
    }

    pub fn time_scale(&self) -> f32 {
        self.sliders[3].value
    }

    pub fn explosion_power(&self) -> f32 {
        self.sliders[4].value
    }

    /// Panel "pożera" kliknięcie, żeby nie tworzyć piłki podczas obsługi UI.
    pub fn contains(&self, point: Vec2) -> bool {
        self.panel.contains(point)
    }

    pub fn set_status(&mut self, message: &str) {
        self.status = Some((message.to_string(), 2.5));
    }

    pub fn tick(&mut self, dt: f32) {
        if let Some((_, remaining)) = &mut self.status {
            *remaining -= dt;
            if *remaining <= 0.0 {
                self.status = None;
            }
        }
    }

    pub fn update(&mut self, mouse: Vec2, pressed: bool, down: bool) -> HudActions {
        let mut actions = HudActions::default();

        self.hover_slider = self
            .sliders
            .iter()
            .position(|slider| slider.hit_area().contains(mouse));
        self.hover_button = self
            .buttons
            .iter()
            .position(|button| button.rect.contains(mouse));

        if pressed {
            self.dragging = None;
            if let Some(index) = self.hover_slider {
                self.dragging = Some(index);
                self.sliders[index].set_from_mouse(mouse);
            } else if let Some(index) = self.hover_button {
                match index {
                    0 => actions.clear = true,
                    1 => actions.add_20 = true,
                    2 => actions.toggle_explosion = true,
                    3 => actions.toggle_vectors = true,
                    4 => actions.toggle_pause = true,
                    5 => actions.save_settings = true,
                    6 => actions.reset_settings = true,
                    _ => {}
                }
            }
        }

        if down {
            if let Some(index) = self.dragging {
                self.sliders[index].set_from_mouse(mouse);
            }
        } else {
            self.dragging = None;
        }

        actions
    }

    pub fn draw(&self, stats: &Stats) {
        let panel = self.panel;
        draw_rectangle(
            panel.x,
            panel.y,
            panel.w,
            panel.h,
            Color::new(0.045, 0.065, 0.09, 0.88),
        );
        draw_rectangle_lines(
            panel.x,
            panel.y,
            panel.w,
            panel.h,
            2.0,
            Color::new(0.32, 0.58, 0.64, 0.55),
        );

        draw_text(
            "PANEL STEROWANIA",
            panel.x + 14.0,
            panel.y + 26.0,
            18.0,
            Color::new(0.65, 0.95, 0.98, 1.0),
        );
        draw_line(
            panel.x + 14.0,
            panel.y + 38.0,
            panel.x + panel.w - 14.0,
            panel.y + 38.0,
            1.0,
            Color::new(0.4, 0.6, 0.65, 0.35),
        );

        let accent = Color::new(0.30, 0.90, 0.88, 1.0);
        let filled = Color::new(0.13, 0.46, 0.50, 1.0);
        let track_color = Color::new(0.14, 0.18, 0.23, 1.0);

        for (index, slider) in self.sliders.iter().enumerate() {
            let label_y = slider.track.y - 14.0;
            draw_text(
                slider.label,
                slider.track.x,
                label_y,
                14.0,
                Color::new(0.80, 0.86, 0.90, 1.0),
            );

            let t = slider.fraction();
            let tr = slider.track;
            draw_rectangle(tr.x, tr.y, tr.w, tr.h, track_color);
            draw_rectangle(tr.x, tr.y, tr.w * t, tr.h, filled);

            let knob_y = tr.y + tr.h * 0.5;
            let knob_x = tr.x + tr.w * t;
            let knob_color = if self.dragging == Some(index) || self.hover_slider == Some(index) {
                Color::new(0.75, 1.0, 0.95, 1.0)
            } else {
                accent
            };
            draw_circle(knob_x, knob_y, 7.5, knob_color);
            draw_circle_lines(knob_x, knob_y, 7.5, 1.5, Color::new(0.0, 0.0, 0.0, 0.45));

            draw_text(
                &slider.value_text(),
                tr.x + tr.w + 10.0,
                label_y,
                14.0,
                Color::new(0.95, 0.97, 1.0, 1.0),
            );
        }

        draw_line(
            panel.x + 14.0,
            panel.y + 310.0,
            panel.x + panel.w - 14.0,
            panel.y + 310.0,
            1.0,
            Color::new(0.4, 0.6, 0.65, 0.35),
        );

        for (index, button) in self.buttons.iter().enumerate() {
            let hovered = self.hover_button == Some(index);
            let armed_button = index == 2 && stats.explosion_armed;
            let r = button.rect;
            let bg = if armed_button {
                Color::new(0.44, 0.25, 0.10, 1.0)
            } else if hovered {
                Color::new(0.16, 0.33, 0.39, 1.0)
            } else {
                Color::new(0.10, 0.17, 0.22, 1.0)
            };
            draw_rectangle(r.x, r.y, r.w, r.h, bg);
            draw_rectangle_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                1.5,
                Color::new(
                    1.0,
                    0.55,
                    0.20,
                    if armed_button || hovered { 0.95 } else { 0.45 },
                ),
            );

            let text = match index {
                2 if stats.explosion_armed => "Cel: ON".to_string(),
                3 => format!(
                    "Wektory: {}",
                    if stats.show_velocity { "ON" } else { "OFF" }
                ),
                4 => format!("Pauza: {}", if stats.paused { "ON" } else { "OFF" }),
                _ => button.label.to_string(),
            };
            draw_text(
                &text,
                r.x + 9.0,
                r.y + 22.0,
                14.0,
                Color::new(0.93, 0.96, 1.0, 1.0),
            );
        }

        let hint_color = Color::new(0.62, 0.70, 0.76, 0.9);
        draw_text(
            "LPM: klik = pilka / przeciaganie = proca",
            panel.x + 14.0,
            panel.y + 450.0,
            12.0,
            hint_color,
        );
        draw_text(
            "PPM: chwyt/rzut | E: eksplozja pod kursorem",
            panel.x + 14.0,
            panel.y + 470.0,
            12.0,
            hint_color,
        );
        draw_text(
            "Spacja: pauza | R: reset | Esc: anuluj cel",
            panel.x + 14.0,
            panel.y + 490.0,
            12.0,
            hint_color,
        );
        draw_text(
            "Moc eksplozji reguluje suwak powyzej",
            panel.x + 14.0,
            panel.y + 510.0,
            12.0,
            Color::new(0.45, 0.75, 0.78, 0.9),
        );

        if let Some((message, _)) = &self.status {
            draw_text(
                message,
                panel.x + 14.0,
                panel.y + 532.0,
                13.0,
                Color::new(1.0, 0.80, 0.40, 1.0),
            );
        }

        // Statystyki w prawym górnym rogu.
        let sx = screen_width() - 224.0;
        draw_rectangle(sx, 12.0, 212.0, 88.0, Color::new(0.045, 0.065, 0.09, 0.82));
        draw_rectangle_lines(
            sx,
            12.0,
            212.0,
            88.0,
            1.5,
            Color::new(0.32, 0.58, 0.64, 0.45),
        );
        draw_text(
            "STATUS",
            sx + 12.0,
            32.0,
            16.0,
            Color::new(0.65, 0.95, 0.98, 1.0),
        );
        draw_text(
            &format!("FPS           {:>5.0}", stats.fps),
            sx + 12.0,
            54.0,
            14.0,
            WHITE,
        );
        draw_text(
            &format!("Pilek         {:>5}", stats.ball_count),
            sx + 12.0,
            72.0,
            14.0,
            WHITE,
        );
        draw_text(
            &format!("Energia       {:>5}", format_energy(stats.energy)),
            sx + 12.0,
            90.0,
            14.0,
            WHITE,
        );

        if stats.paused {
            let width = screen_width();
            let height = screen_height();
            draw_rectangle(0.0, 0.0, width, height, Color::new(0.0, 0.0, 0.0, 0.22));
            let dims = measure_text("PAUZA", None, 44, 1.0);
            draw_text(
                "PAUZA",
                (width - dims.width) * 0.5,
                height * 0.5,
                44.0,
                Color::new(1.0, 1.0, 1.0, 0.92),
            );
        }

        // Podpowiedź na dole ekranu.
        let help = "LPM klik: pilka  |  LPM przeciaganie: proca  |  PPM: chwyt/rzut  |  E: eksplozja  |  Spacja: pauza  |  R: reset";
        let dims = measure_text(help, None, 13, 1.0);
        draw_text(
            help,
            (screen_width() - dims.width) * 0.5,
            screen_height() - 12.0,
            13.0,
            Color::new(0.72, 0.80, 0.85, 0.65),
        );
    }
}

impl Default for Hud {
    fn default() -> Self {
        Self::new()
    }
}

fn format_energy(value: f32) -> String {
    if value >= 1.0e9 {
        format!("{:.2}G", value / 1.0e9)
    } else if value >= 1.0e6 {
        format!("{:.2}M", value / 1.0e6)
    } else if value >= 1.0e3 {
        format!("{:.1}k", value / 1.0e3)
    } else {
        format!("{:.0}", value)
    }
}
