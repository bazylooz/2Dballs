//! Trwałe ustawienia aplikacji - prosty plik tekstowy, bez dodatkowych zależności.

use std::path::{Path, PathBuf};

pub const GRAVITY_RANGE: (f32, f32) = (0.0, 2000.0);
pub const RESTITUTION_RANGE: (f32, f32) = (0.0, 1.0);
pub const AIR_DRAG_RANGE: (f32, f32) = (0.0, 2.0);
pub const TIME_SCALE_RANGE: (f32, f32) = (0.1, 2.0);
pub const EXPLOSION_POWER_RANGE: (f32, f32) = (200.0, 4000.0);

pub const SETTINGS_FILE: &str = "physics_settings.txt";

#[derive(Clone, Debug)]
pub struct AppSettings {
    pub gravity: f32,
    pub restitution: f32,
    pub air_drag: f32,
    pub time_scale: f32,
    pub explosion_power: f32,
    pub show_velocity: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            gravity: 900.0,
            restitution: 0.86,
            air_drag: 0.10,
            time_scale: 1.0,
            explosion_power: 1700.0,
            show_velocity: false,
        }
    }
}

impl AppSettings {
    pub fn path() -> PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(SETTINGS_FILE)
    }

    /// Wczytanie ustawień; przy braku pliku lub błędzie zwracane są domyślne.
    pub fn load() -> Self {
        Self::load_from(&Self::path())
    }

    /// Wczytanie ustawień z dowolnej ścieżki (użyteczne także w testach).
    pub fn load_from(path: &Path) -> Self {
        let mut settings = Self::default();
        let Ok(text) = std::fs::read_to_string(path) else {
            return settings;
        };

        for line in text.lines() {
            let Some((key, raw_value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = raw_value.trim();
            match key {
                "gravity" => {
                    if let Ok(v) = value.parse::<f32>() {
                        settings.gravity = clamp(v, GRAVITY_RANGE);
                    }
                }
                "restitution" => {
                    if let Ok(v) = value.parse::<f32>() {
                        settings.restitution = clamp(v, RESTITUTION_RANGE);
                    }
                }
                "air_drag" => {
                    if let Ok(v) = value.parse::<f32>() {
                        settings.air_drag = clamp(v, AIR_DRAG_RANGE);
                    }
                }
                "time_scale" => {
                    if let Ok(v) = value.parse::<f32>() {
                        settings.time_scale = clamp(v, TIME_SCALE_RANGE);
                    }
                }
                "explosion_power" => {
                    if let Ok(v) = value.parse::<f32>() {
                        settings.explosion_power = clamp(v, EXPLOSION_POWER_RANGE);
                    }
                }
                "show_velocity" => {
                    settings.show_velocity = value == "1" || value.eq_ignore_ascii_case("true");
                }
                _ => {}
            }
        }
        settings
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.save_to(&Self::path())
    }

    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        let text = format!(
            "# Ustawienia symulatora fizyki 2D\n\
             gravity={:.2}\n\
             restitution={:.3}\n\
             air_drag={:.3}\n\
             time_scale={:.3}\n\
             explosion_power={:.1}\n\
             show_velocity={}\n",
            self.gravity,
            self.restitution,
            self.air_drag,
            self.time_scale,
            self.explosion_power,
            if self.show_velocity { 1 } else { 0 },
        );
        std::fs::write(path, text)
    }
}

fn clamp(value: f32, range: (f32, f32)) -> f32 {
    if value.is_finite() {
        value.clamp(range.0, range.1)
    } else {
        range.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("ball_physics_{}_{}.txt", name, std::process::id()))
    }

    #[test]
    fn settings_roundtrip() {
        let path = test_path("roundtrip");
        let original = AppSettings {
            gravity: 1234.0,
            restitution: 0.5,
            air_drag: 0.75,
            time_scale: 1.5,
            explosion_power: 2500.0,
            show_velocity: true,
        };
        original.save_to(&path).expect("zapis ustawień");
        let loaded = AppSettings::load_from(&path);

        assert_eq!(loaded.gravity, original.gravity);
        assert_eq!(loaded.restitution, original.restitution);
        assert_eq!(loaded.air_drag, original.air_drag);
        assert_eq!(loaded.time_scale, original.time_scale);
        assert_eq!(loaded.explosion_power, original.explosion_power);
        assert_eq!(loaded.show_velocity, original.show_velocity);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn missing_file_returns_defaults_and_bad_values_are_clamped() {
        let path = test_path("invalid");
        let _ = std::fs::remove_file(&path);
        let defaults = AppSettings::load_from(&path);
        assert_eq!(defaults.gravity, AppSettings::default().gravity);

        std::fs::write(
            &path,
            "gravity=99999\nrestitution=-4\nexplosion_power=not-a-number\nshow_velocity=true\n",
        )
        .expect("zapis pliku testowego");
        let loaded = AppSettings::load_from(&path);
        assert_eq!(loaded.gravity, GRAVITY_RANGE.1);
        assert_eq!(loaded.restitution, RESTITUTION_RANGE.0);
        assert_eq!(
            loaded.explosion_power,
            AppSettings::default().explosion_power
        );
        assert!(loaded.show_velocity);
        std::fs::remove_file(path).ok();
    }
}
