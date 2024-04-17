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
        }

        // Draw all bodies & trails.
        clear_background(BLACK);
        bodies.iter().for_each(Body::draw);
        trails.iter().for_each(Trail::draw);

        // If two bodies collide, stop the simulation.
        if has_collision(&bodies) {
            running = false;
        }

        next_frame().await
    }
}

fn has_collision(bodies: &[Body]) -> bool {
    bodies.iter().any(|body| {
        bodies
            .iter()
            .any(|other| body.id != other.id && body.position.distance(other.position) < 10.0)
    })
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

    fn update_velocity<'a>(&mut self, others: impl Iterator<Item = Self>) {
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

    fn update_position(&mut self) {
        self.position += self.velocity;
    }
}

#[derive(Clone, Copy)]
struct Trail {
    position: Vec2,
    colour: Color,
}

impl Trail {
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
