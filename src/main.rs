use macroquad::prelude::*;

#[macroquad::main("Three Bodies")]
async fn main() {
    rand::srand(42);
    let mut bodies = [
        Body::new_random(0),
        Body::new_random(1),
        Body::new_random(2),
    ];
    let mut running = true;

    loop {
        clear_background(BLACK);

        // Reset on space.
        if is_key_released(KeyCode::Space) {
            bodies = [
                Body::new_random(0),
                Body::new_random(1),
                Body::new_random(2),
            ];
            running = true;
        }

        if running {
            // Calculate forces to apply based on last frame's positions.
            let mut new_bodies = bodies;
            for body in new_bodies.iter_mut() {
                body.velocity += bodies
                    .iter()
                    .filter(|&other| other.id != body.id)
                    .map(|other| {
                        let delta = other.position - body.position;
                        let distance = delta.length();
                        let direction = delta.normalize();
                        let force = (body.mass * other.mass) / (distance * distance);
                        direction * force
                    })
                    .reduce(|acc, force| acc + force)
                    .unwrap();
            }

            // Update positions based on new velocities and redraw.
            bodies = new_bodies;
            bodies.iter_mut().for_each(|body| body.update_position());
        }

        // Draw all bodies.
        bodies.iter().for_each(|body| body.draw());

        // If two bodies collide, stop the simulation.
        if bodies.iter().any(|body| {
            bodies
                .iter()
                .any(|other| body.id != other.id && body.position.distance(other.position) < 10.0)
        }) {
            running = false;
        }

        next_frame().await
    }
}

#[derive(Clone, Copy)]
struct Body {
    id: usize,
    colour: Color,
    position: Vec2,
    velocity: Vec2,
    mass: f32,
}

impl Body {
    fn new_random(id: usize) -> Self {
        let colour = Color::new(
            rand::gen_range(0.0, 1.0),
            rand::gen_range(0.0, 1.0),
            rand::gen_range(0.0, 1.0),
            1.0,
        );
        let position = vec2(
            rand::gen_range(100., screen_width() - 100.),
            rand::gen_range(100., screen_height() - 100.),
        );
        let velocity = vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0));
        let mass = rand::gen_range(5., 10.);
        Self {
            id,
            colour,
            position,
            velocity,
            mass,
        }
    }

    fn draw(&self) {
        draw_circle(self.position.x, self.position.y, 5.0, self.colour);
    }

    fn update_position(&mut self) {
        self.position += self.velocity;
    }
}
