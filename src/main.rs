//! Natywny symulator 2D fizyki piłek - własny silnik fizyki + macroquad.
//!
//! Sterowanie:
//!   LPM (klik)       - nowa piłka o losowym promieniu i kolorze
//!   LPM (przeciągnij)- proca: wystrzał w przeciwną stronę do przeciągnięcia
//!   PPM              - chwyć piłkę i rzuć nią (pęd z ruchu kursora)
//!   Spacja           - pauza / wznowienie
//!   R                - reset planszy

mod ball;
mod effects;
mod physics;
mod settings;
mod ui;

use macroquad::prelude::*;

use ball::random_ball;
use effects::ExplosionFx;
use physics::{PhysicsConfig, World};
use settings::AppSettings;
use ui::{Hud, Stats};

fn window_conf() -> Conf {
    Conf {
        window_title: "2D Physics Sandbox - własny silnik fizyki".to_string(),
        window_width: 1280,
        window_height: 800,
        window_resizable: true,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let saved = AppSettings::load();
    let mut world = World::new(vec2(screen_width(), screen_height()));
    world.config.gravity = saved.gravity;
    world.config.restitution = saved.restitution;
    world.config.air_drag = saved.air_drag;
    world.time_scale = saved.time_scale;
    world.reset();

    let mut hud = Hud::new();
    hud.apply_settings(&saved);
    let mut paused = false;
    let mut show_velocity = saved.show_velocity;
    let mut explosion_armed = false;
    let mut effects: Vec<ExplosionFx> = Vec::new();

    let mut last_mouse: Vec2 = mouse_position().into();
    let mut grabbed: Option<usize> = None;
    let mut sling_start: Option<Vec2> = None;

    loop {
        let mouse: Vec2 = mouse_position().into();
        let frame_time = get_frame_time();
        let left_pressed = is_mouse_button_pressed(MouseButton::Left);
        let left_down = is_mouse_button_down(MouseButton::Left);
        let left_released = is_mouse_button_released(MouseButton::Left);
        let right_pressed = is_mouse_button_pressed(MouseButton::Right);
        let right_down = is_mouse_button_down(MouseButton::Right);
        let right_released = is_mouse_button_released(MouseButton::Right);

        // Płynne dostosowanie planszy do rozmiaru okna.
        let width = screen_width();
        let height = screen_height();
        if (width - world.config.bounds.x).abs() > 0.5
            || (height - world.config.bounds.y).abs() > 0.5
        {
            world.resize(width, height);
        }

        // Synchronizacja parametrów z panelem.
        world.config.gravity = hud.gravity();
        world.config.restitution = hud.restitution();
        world.config.air_drag = hud.air_drag();
        world.time_scale = hud.time_scale();

        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
            world.flush();
        }
        if is_key_pressed(KeyCode::R) {
            world.reset();
            grabbed = None;
            sling_start = None;
        }
        if is_key_pressed(KeyCode::Escape) && explosion_armed {
            explosion_armed = false;
            hud.set_status("Cel eksplozji anulowany");
        }
        if is_key_pressed(KeyCode::E) {
            let center = if hud.contains(mouse) {
                world.config.bounds * 0.5
            } else {
                mouse
            };
            detonate(&mut world, &mut effects, center, hud.explosion_power());
            hud.set_status("Eksplozja!");
        }

        hud.tick(frame_time);
        let actions = hud.update(mouse, left_pressed, left_down);
        if actions.clear {
            world.clear();
            grabbed = None;
            sling_start = None;
        }
        if actions.add_20 {
            world.add_random(20);
        }
        if actions.toggle_explosion {
            explosion_armed = !explosion_armed;
            hud.set_status(if explosion_armed {
                "Kliknij w planszy, aby wysadzic"
            } else {
                "Cel eksplozji anulowany"
            });
        }
        if actions.toggle_vectors {
            show_velocity = !show_velocity;
        }
        if actions.toggle_pause {
            paused = !paused;
            world.flush();
        }
        if actions.save_settings {
            match hud.current_settings(show_velocity).save() {
                Ok(()) => hud.set_status("Ustawienia zapisane"),
                Err(error) => hud.set_status(&format!("Blad zapisu: {}", error)),
            }
        }
        if actions.reset_settings {
            let defaults = AppSettings::default();
            hud.apply_settings(&defaults);
            show_velocity = defaults.show_velocity;
            let message = match defaults.save() {
                Ok(()) => "Ustawienia domyslne (zapisane)",
                Err(_) => "Ustawienia domyslne",
            };
            hud.set_status(message);
        }

        let over_ui = hud.contains(mouse);

        if right_pressed && explosion_armed {
            explosion_armed = false;
            hud.set_status("Cel eksplozji anulowany");
        }

        // PPM - chwytanie i rzucanie piłką.
        if right_pressed && !over_ui && grabbed.is_none() {
            if let Some(index) = world.pick_ball(mouse) {
                grabbed = Some(index);
                world.balls[index].set_held(true);
                sling_start = None;
            }
        }
        if let Some(index) = grabbed {
            if right_down {
                let ball = &mut world.balls[index];
                let r = ball.radius;
                ball.pos.x = mouse.x.clamp(r, (width - r).max(r));
                ball.pos.y = mouse.y.clamp(r, (height - r).max(r));
                let raw_velocity = (mouse - last_mouse) / frame_time.max(1.0 / 240.0);
                ball.vel = raw_velocity.clamp_length_max(2500.0);
            }
            if right_released {
                world.balls[index].set_held(false);
                grabbed = None;
            }
        }

        // LPM - eksplozja w wybranym miejscu, proca lub szybkie tworzenie piłki.
        let explosion_click = explosion_armed && left_pressed && !over_ui;
        if explosion_click {
            detonate(&mut world, &mut effects, mouse, hud.explosion_power());
            explosion_armed = false;
            sling_start = None;
            hud.set_status("Eksplozja!");
        }
        if left_pressed && !over_ui && grabbed.is_none() && !explosion_click {
            if world.pick_ball(mouse).is_none() {
                sling_start = Some(mouse);
            }
        }
        if let Some(start) = sling_start {
            if left_released {
                let pull = start - mouse;
                let velocity = if pull.length() < 8.0 {
                    vec2(
                        macroquad::rand::gen_range(-120.0_f32, 120.0_f32),
                        macroquad::rand::gen_range(-120.0_f32, 120.0_f32),
                    )
                } else {
                    (pull * 4.2).clamp_length_max(2200.0)
                };
                world
                    .balls
                    .push(random_ball(start, world.config.bounds, velocity));
                sling_start = None;
            }
        }

        // Fizyka ze stałym krokiem - niezależna od FPS.
        if !paused {
            world.advance(frame_time);
            for ball in &mut world.balls {
                ball.trail.push(ball.pos);
            }
        }

        // Animacja eksplozji w czasie rzeczywistym.
        for effect in &mut effects {
            effect.update(frame_time);
        }
        effects.retain(|effect| !effect.finished());

        // Render.
        draw_background();
        for ball in &world.balls {
            ball.draw(show_velocity);
        }
        for effect in &effects {
            effect.draw();
        }
        if let Some(start) = sling_start {
            draw_slingshot(start, mouse, &world.config);
        }
        if explosion_armed && !over_ui {
            draw_explosion_target(mouse, hud.explosion_power());
        }

        let stats = Stats {
            fps: get_fps() as f32,
            ball_count: world.balls.len(),
            energy: world.kinetic_energy(),
            paused,
            show_velocity,
            explosion_armed,
        };
        hud.draw(&stats);

        last_mouse = mouse;
        next_frame().await;
    }
}

fn detonate(world: &mut World, effects: &mut Vec<ExplosionFx>, center: Vec2, power: f32) {
    world.explode_at(center, power);
    effects.push(ExplosionFx::new(center, power));
}

fn draw_explosion_target(mouse: Vec2, power: f32) {
    let bright = Color::new(1.0, 0.55, 0.20, 0.95);
    let dim = Color::new(1.0, 0.55, 0.20, 0.45);
    draw_circle_lines(mouse.x, mouse.y, 18.0, 2.0, bright);
    draw_circle_lines(mouse.x, mouse.y, 30.0, 1.0, dim);
    draw_line(
        mouse.x - 38.0,
        mouse.y,
        mouse.x - 24.0,
        mouse.y,
        2.0,
        bright,
    );
    draw_line(
        mouse.x + 24.0,
        mouse.y,
        mouse.x + 38.0,
        mouse.y,
        2.0,
        bright,
    );
    draw_line(
        mouse.x,
        mouse.y - 38.0,
        mouse.x,
        mouse.y - 24.0,
        2.0,
        bright,
    );
    draw_line(
        mouse.x,
        mouse.y + 24.0,
        mouse.x,
        mouse.y + 38.0,
        2.0,
        bright,
    );
    draw_text(
        &format!("Moc {:.0}", power),
        mouse.x + 22.0,
        mouse.y - 20.0,
        14.0,
        bright,
    );
}

fn draw_background() {
    clear_background(Color::new(0.035, 0.045, 0.065, 1.0));

    let width = screen_width();
    let height = screen_height();
    let step = 48.0;
    let minor = Color::new(1.0, 1.0, 1.0, 0.035);
    let major = Color::new(0.35, 0.75, 0.85, 0.075);

    let mut x = 0.0;
    let mut column = 0;
    while x <= width {
        let color = if column % 4 == 0 { major } else { minor };
        draw_line(x, 0.0, x, height, 1.0, color);
        x += step;
        column += 1;
    }

    let mut y = 0.0;
    let mut row = 0;
    while y <= height {
        let color = if row % 4 == 0 { major } else { minor };
        draw_line(0.0, y, width, y, 1.0, color);
        y += step;
        row += 1;
    }

    draw_rectangle_lines(
        0.0,
        0.0,
        width,
        height,
        2.0,
        Color::new(0.35, 0.75, 0.85, 0.30),
    );
}

fn draw_slingshot(start: Vec2, current: Vec2, config: &PhysicsConfig) {
    let pull = start - current;
    let distance = pull.length();
    if distance < 2.0 {
        return;
    }

    let direction = pull / distance;
    let segment = 12.0;
    let gap = 8.0;
    let mut d = 0.0;
    while d < distance {
        let a = start + direction * d;
        let b = start + direction * (d + segment).min(distance);
        draw_line(a.x, a.y, b.x, b.y, 2.0, Color::new(1.0, 0.78, 0.30, 0.85));
        d += segment + gap;
    }

    draw_circle_lines(
        start.x,
        start.y,
        10.0,
        2.0,
        Color::new(0.35, 0.95, 0.90, 0.9),
    );
    draw_circle(current.x, current.y, 4.0, Color::new(1.0, 0.78, 0.30, 0.9));

    // Podgląd trajektorii (ten sam model grawitacji i oporu co w fizyce).
    let mut velocity = (pull * 4.2).clamp_length_max(2200.0);
    let mut position = start;
    let dt = 1.0 / 60.0;
    for i in 0..42 {
        velocity.y += config.gravity * dt;
        velocity *= (-config.air_drag * dt).exp();
        position += velocity * dt;
        if position.x < 0.0
            || position.x > config.bounds.x
            || position.y < 0.0
            || position.y > config.bounds.y
        {
            break;
        }
        let alpha = 0.55 * (1.0 - i as f32 / 42.0);
        draw_circle(
            position.x,
            position.y,
            2.5,
            Color::new(1.0, 0.90, 0.50, alpha),
        );
    }

    let power = (distance / 240.0 * 100.0).min(100.0);
    draw_text(
        &format!("Moc: {:.0}%", power),
        current.x + 12.0,
        current.y - 12.0,
        14.0,
        Color::new(1.0, 0.85, 0.45, 0.95),
    );
}
