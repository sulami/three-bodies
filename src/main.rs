use std::collections::VecDeque;

use macroquad::prelude::*;

#[cfg(target_arch = "wasm32")]
const IS_WASM: bool = true;
#[cfg(not(target_arch = "wasm32"))]
const IS_WASM: bool = false;

#[macroquad::main("Three Bodies")]
async fn main() {
    rand::srand(42);
    let mut bodies = [
        Body::new_random(0),
        Body::new_random(1),
        Body::new_random(2),
    ];
    let mut trails: VecDeque<Trail> = VecDeque::new();
    let mut running = true;
    let mut show_ui = Ui::Full;
    let mut auto_restart = IS_WASM;
    let mut elastic_collisions = false;

    loop {
        // Exit on escape.
        if !IS_WASM && is_key_released(KeyCode::Escape) {
            break;
        }

        // Reset on space, or if auto restart is on.
        if is_key_released(KeyCode::Space)
            || is_mouse_button_released(MouseButton::Left)
            || (!running && auto_restart)
        {
            bodies = [
                Body::new_random(0),
                Body::new_random(1),
                Body::new_random(2),
            ];
            trails.clear();
            running = true;
        }

        // Toggle UI on U.
        if is_key_released(KeyCode::U) {
            show_ui.toggle();
        }

        // Toggle auto-restart on R.
        if is_key_released(KeyCode::R) {
            auto_restart = !auto_restart;
        }

        // Toggle elastic collisions on C.
        if is_key_released(KeyCode::C) {
            elastic_collisions = !elastic_collisions;
        }

        if running {
            // Calculate forces to apply based on last frame's positions.
            let mut new_bodies = bodies;
            new_bodies.iter_mut().for_each(|body| {
                body.update_velocity(bodies.iter().copied(), elastic_collisions);
            });

            // Update positions based on new velocities.
            bodies = new_bodies;
            trails.iter_mut().for_each(|trail| trail.colour.a *= 0.995);
            trails.extend(bodies.iter().map(Trail::from));
            while trails.front().map_or(false, |trail| trail.colour.a < 0.01) {
                trails.pop_front();
            }
            bodies.iter_mut().for_each(Body::update_position);

            if !elastic_collisions {
                // If two bodies collide, stop the simulation.
                running = !has_collision(&bodies);
            }
        }

        // Draw all bodies & trails.
        clear_background(BLACK);
        bodies.iter().for_each(Body::draw);
        trails.iter().for_each(Trail::draw);
        draw_ui(&bodies, show_ui, auto_restart, running, elastic_collisions);

        next_frame().await
    }
}

/// Returns true if any two bodies are colliding.
fn has_collision(bodies: &[Body]) -> bool {
    for i in 0..bodies.len() {
        for j in i + 1..bodies.len() {
            if bodies[i].collides_with(&bodies[j]) {
                return true;
            }
        }
    }
    false
}

/// Draws the UI.
fn draw_ui(
    bodies: &[Body],
    show_ui: Ui,
    auto_restart: bool,
    running: bool,
    elastic_collisions: bool,
) {
    if !running {
        draw_text(
            "COLLISION",
            screen_width() / 2.0 - 64.0, // NB Manually centred.
            screen_height() / 2.0,
            32.0,
            WHITE,
        );
    }

    // Body info
    if matches!(show_ui, Ui::Full | Ui::Minimal) {
        for body in bodies {
            draw_text(
                &format!("m {:.2}", body.mass),
                body.position.x + 10.0,
                body.position.y + 10.0,
                16.0,
                body.colour,
            );
            draw_text(
                &format!("v {:.2}", body.velocity.length()),
                body.position.x + 10.0,
                body.position.y + 20.0,
                16.0,
                body.colour,
            );
        }
    }

    // Instructions
    if matches!(show_ui, Ui::Full) {
        let instructions = [
            "[SPACE/CLICK/TAP] reset",
            "[U] toggle UI",
            &format!(
                "[R] toggle auto-restart ({})",
                if auto_restart { "on" } else { "off" }
            ),
            &format!(
                "[C] toggle elastic collisions ({})",
                if elastic_collisions { "on" } else { "off" }
            ),
        ];
        instructions
            .iter()
            .enumerate()
            .for_each(|(idx, instruction)| {
                draw_text(
                    instruction,
                    10.0,
                    screen_height() - 14.0 - idx as f32 * 14.0,
                    16.0,
                    WHITE,
                )
            });
    }
}

#[derive(Clone, Copy)]
enum Ui {
    Full,
    Minimal,
    Off,
}

impl Ui {
    /// Toggles to the next UI state.
    fn toggle(&mut self) {
        *self = match self {
            Ui::Off => Ui::Minimal,
            Ui::Minimal => Ui::Full,
            Ui::Full => Ui::Off,
        }
    }
}

/// A body in the simulation.
#[derive(Clone, Copy)]
struct Body {
    id: usize,
    colour: Color,
    position: Vec2,
    velocity: Vec2,
    mass: f32,
}

impl Body {
    /// Creates a new body with random properties.
    fn new_random(id: usize) -> Self {
        let colour = Color::new(
            rand::gen_range(0.2, 1.0),
            rand::gen_range(0.2, 1.0),
            rand::gen_range(0.2, 1.0),
            1.0,
        );
        let position = vec2(
            rand::gen_range(screen_width() * 0.25, screen_width() * 0.75),
            rand::gen_range(screen_height() * 0.25, screen_height() * 0.75),
        );
        let velocity = vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0));
        let mass = rand::gen_range(1., 10.);
        Self {
            id,
            colour,
            position,
            velocity,
            mass,
        }
    }

    /// Draws the body on the screen.
    fn draw(&self) {
        draw_circle(self.position.x, self.position.y, self.mass, self.colour);
    }

    /// Updates the velocity of the body based on the forces applied by other bodies.
    fn update_velocity(
        &mut self,
        bodies: impl Iterator<Item = Self> + Clone,
        elastic_collisions: bool,
    ) {
        let mut collided = elastic_collisions;
        if elastic_collisions {
            self.velocity = bodies
                .clone()
                .filter(|&body| body.id != self.id)
                .filter(|other| self.collides_with(other))
                .map(|other| {
                    let m1 = self.mass;
                    let m2 = other.mass;
                    let v1 = self.velocity;
                    let v2 = other.velocity;
                    let v1_prime = ((m1 - m2) / (m1 + m2)) * v1 + ((2.0 * m2) / (m1 + m2)) * v2;
                    v1_prime
                })
                .reduce(|acc, velocity| acc + velocity)
                .unwrap_or_else(|| {
                    collided = false;
                    self.velocity
                });
        }
        if collided {
            return;
        }
        self.velocity += bodies
            .filter(|&body| body.id != self.id)
            .map(|other| {
                let mut delta = other.position - self.position;
                if delta.x.abs() > screen_width() / 2.0 {
                    delta.x = delta.x - delta.x.signum() * screen_width();
                }

                if delta.y.abs() > screen_height() / 2.0 {
                    delta.y = delta.y - delta.y.signum() * screen_height();
                }
                let distance = delta.length();
                let direction = delta.normalize();
                let force = (self.mass * other.mass) / (distance * distance);
                direction * force
            })
            .reduce(|acc, force| acc + force)
            .map(|force| 9.81 * force / self.mass)
            .unwrap();
    }

    /// Updates the position of the body based on its velocity.
    fn update_position(&mut self) {
        self.position += self.velocity;
        if self.position.x > screen_width() {
            self.position.x -= screen_width();
        } else if self.position.x < 0. {
            self.position.x += screen_width();
        }
        if self.position.y > screen_height() {
            self.position.y -= screen_height();
        } else if self.position.y < 0. {
            self.position.y += screen_height();
        }
    }

    /// Returns true if this body collides with another.
    fn collides_with(&self, other: &Self) -> bool {
        self.position.distance(other.position) <= self.mass + other.mass
    }
}

/// A trail left behind by a body.
#[derive(Clone, Copy)]
struct Trail {
    position: Vec2,
    colour: Color,
}

impl Trail {
    /// Draws the trail on the screen.
    fn draw(&self) {
        draw_circle(self.position.x, self.position.y, 1.0, self.colour);
    }
}

impl From<&Body> for Trail {
    fn from(body: &Body) -> Self {
        Self {
            position: body.position,
            colour: body.colour,
        }
    }
}
