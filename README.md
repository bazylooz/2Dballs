# 2D Physics Simulator - Custom Engine + macroquad

A native Rust application featuring a custom-built 2D physics engine (with no external physics libraries). It handles gravity, air resistance, rolling friction, wall collisions, and elastic ball-to-ball collisions with momentum conservation and positional correction.

## Running the Project

Rust (cargo) is required; the project was built using version 1.100.

```bash
cargo build --release
./target/release/ball_physics_sim      # Linux / macOS
target\release\ball_physics_sim.exe    # Windows
```

Development build (faster compilation, slower runtime performance):

```bash
cargo run
```

## Controls

| Action | Description |
| --- | --- |
| **LMB** (click empty space) | Spawns a new ball with a random radius and color |
| **LMB** (drag) | Slingshot – aiming line, trajectory preview, and launch on release |
| **RMB** (hold & move) | Grabs a ball and throws it with momentum matching mouse movement |
| **E** | Triggers an explosion at the cursor (animation + radial impulse) |
| **Explosion button** | Arms target mode – the next LMB click on the canvas detonates at that location |
| **Esc** / **RMB** | Cancels armed explosion target |
| **Space** | Pauses / resumes simulation |
| **R** | Resets the board |

## Control Panel (Top-Left Corner)

Sliders:

- **Gravity** – 0 … 2000 px/s²
- **Restitution** – 0 … 1 (bounciness)
- **Air Drag** – 0 … 2 1/s
- **Time Scale** – 0.1x … 2.0x (slow motion / fast-forward)
- **Explosion Power** – 200 … 4000 (radial impulse strength)

Buttons: **Clear**, **+20 Balls**, **Explosion** (arms target), **Vectors** (velocity vectors), **Pause**, **Save**, **Defaults**.

Settings are saved to `physics_settings.txt` in the working directory and loaded automatically on startup. Clicking **Defaults** restores the initial values and saves them as well. A brief status notification (e.g., *"Settings saved"*) appears after each panel action.

The top-right corner displays FPS, ball count, and total kinetic energy.

## Code Structure

- `src/main.rs` – entry point, time accumulator, input handling, explosion controls, rendering
- `src/ball.rs` – ball struct, mass ∝ πr², trail ring buffer, rendering
- `src/physics.rs` – physics engine: integration, collisions, spatial grid, impulses
- `src/effects.rs` – explosion animation (rings, flash, rays)
- `src/settings.rs` – settings serialization/deserialization and default values
- `src/ui.rs` – custom HUD (sliders, buttons, statistics)

The physics simulation runs at a fixed 120 Hz timestep (using an accumulator capped at 16 substeps per frame), ensuring deterministic behavior independent of the current FPS. Explosion animations play in real time regardless of the simulation's time scale.
