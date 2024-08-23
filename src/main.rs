use cgmath::{Vector2, Vector3, Zero};
use winit::dpi::PhysicalSize;
use game_engine::engine::physics_engine::collision::collision_body::CollisionBody;
use game_engine::engine::{physics_engine::integrator::verlet::VerletIntegrator, run::run, Simulation};

struct FlappyBird {
    integrator: VerletIntegrator,
    dt: f32,
    window_size: Vector2<f32>, 
}

const PIPE_WIDTH: f32 = 250.0;
const PIPE_PAIR_DISTANCE_Y: f32 = 150.0;

impl FlappyBird {
    fn new(window_size: &PhysicalSize<u32>) -> Self {
        let dt = 0.001;
        let gravity: f32 = 1000000.0;
        let accelleration = Vector3::new(0.0, -gravity, 0.0);

        let player = CollisionBody::circle(0, Vector3::zero(), accelleration, 
            Vector3::zero(), Vector3::zero(), 50.0, Vector3::new(255.0,0.0,0.0)); 

        // TODO: Create ID incrementer
        let (top_pipe, bot_pipe) = Self::create_pipe_pair(100.0, PIPE_WIDTH,
            PIPE_PAIR_DISTANCE_Y, 0.25, window_size.height as f32, -2.0);

        let integrator = VerletIntegrator::new(f32::MAX, vec![player, top_pipe, bot_pipe]);

        let window_size = Vector2::new(window_size.width as f32, window_size.height as f32);

        Self {
            integrator, dt, window_size,
        }
    }

    fn create_pipe_pair(
        x: f32, pipe_width:f32, hole_size:f32, 
        hole_weight:f32, window_height:f32, velocity_x: f32
    ) -> (CollisionBody, CollisionBody) {
        
        let top_weight = 1.0 - hole_weight;
        let bot_weight = 1.0 + hole_weight;

        let velocity = Vector3::new(velocity_x,0.0,0.0);
        let acceleration = Vector3::zero();
        let color = Vector3::new(0.0,255.0,0.0);

        let top_position = Vector3::new(x, 
            window_height - (window_height-hole_size/2.0)*top_weight, 0.0);
        let top_prev_position = top_position - velocity;
        let top = CollisionBody::rectangle(1, velocity, acceleration, top_prev_position,
            top_position, pipe_width, window_height*2.0, color);

        let bot_position = Vector3::new(x, -window_height, 0.0);
        let bot_prev_position = bot_position - velocity;
        let bot = CollisionBody::rectangle(2, velocity, acceleration, bot_prev_position,
            bot_position, pipe_width,  (window_height - hole_size/2.0)*bot_weight, color) ;

        (top, bot)
    }
}

impl Simulation for FlappyBird {

    fn update(&mut self) {
        self.integrator.update(self.dt);
        let bodies = self.integrator.get_bodies_mut();
        let player = &bodies[0];
        if player.position.y < -self.window_size.y {
            panic!("Game Over!");
        } 
    }

    fn get_bodies(&self) -> &Vec<CollisionBody> {
        &self.integrator.get_bodies()
    }

    fn get_num_active_instances(&self) -> u32 {
        1
    }

    fn get_target_num_instances(&self) -> u32 {
        1
    }

    fn jump(&mut self) {
        self.integrator.set_velocity_y(0, 20.0);
    }
}

fn main() {
    let window_size = PhysicalSize::new(600,600);
    let flappy_bird = FlappyBird::new(&window_size);
    pollster::block_on(run(flappy_bird, window_size, 20));
}
