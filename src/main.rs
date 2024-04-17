use std::fmt::Display;

use macroquad::prelude::*;

#[macroquad::main("Three Bodies")]
async fn main() {
    rand::srand(42);
    let mut bodies = [
        Body::new_random(0),
        Body::new_random(1),
        Body::new_random(2),
    ];
    let mut trails: Vec<Trail> = vec![];
    let mut running = true;
    let mut show_ui = true;

    loop {
        // Exit on escape
        if is_key_released(KeyCode::Escape) {
            break;
        }

        // Reset on space.
        if is_key_released(KeyCode::Space) {
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
            show_ui = !show_ui;
        }

        if running {
            // Calculate forces to apply based on last frame's positions.
            let mut new_bodies = bodies;
            new_bodies.iter_mut().for_each(|body| {
                body.update_velocity(bodies.iter().copied());
            });

            // Update positions based on new velocities.
            bodies = new_bodies;
            trails.extend(bodies.iter().map(Trail::from));
            bodies.iter_mut().for_each(Body::update_position);

            // If two bodies collide, stop the simulation.
            running = !has_collision(&bodies)
        }

        // Draw all bodies & trails.
        clear_background(BLACK);
        bodies.iter().for_each(Body::draw);
        trails.iter().for_each(Trail::draw);
        if show_ui {
            draw_ui(&bodies);
        }

        next_frame().await
    }
}

/// Returns true if any two bodies are colliding.
fn has_collision(bodies: &[Body]) -> bool {
    for i in 0..bodies.len() {
        for j in i + 1..bodies.len() {
            if bodies[i].position.distance(bodies[j].position) < 10.0 {
                return true;
            }
        }
    }
    false
}

/// Draws the UI.
fn draw_ui(bodies: &[Body]) {
    // Body info
    for body in bodies {
        draw_text(
            &format!("{}", body),
            10.0,
            20.0 * (body.id as f32 + 1.0),
            20.0,
            body.colour,
        );
    }

    // Instructions
    draw_text(
        "Press SPACE to reset, ESC to exit, U to toggle UI",
        10.0,
        screen_height() - 10.0,
        20.0,
        WHITE,
    );
}

/// A body in the simulation.
#[derive(Clone, Copy, Debug)]
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
        draw_circle(self.position.x, self.position.y, 5.0, self.colour);
    }

    /// Updates the velocity of the body based on the forces applied by other bodies.
    fn update_velocity(&mut self, others: impl Iterator<Item = Self>) {
        self.velocity += others
            .filter(|&other| other.id != self.id)
            .map(|other| {
                let delta = other.position - self.position;
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
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Body {}: mass {:.2} position ({:.2}, {:.2})",
            self.id, self.mass, self.position.x, self.position.y
        )
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
