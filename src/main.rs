use cgmath::{Vector2, Vector3, Zero};
use game_engine::engine::physics_engine::broadphase::BroadPhase;
use game_engine::engine::physics_engine::collision::collision_handler::IdentityCollisionSolver;
use game_engine::engine::physics_engine::collision::CollisionGraph;
use game_engine::engine::physics_engine::narrowphase::NarrowPhase;
use game_engine::engine::physics_engine::{broadphase, narrowphase};
use rand::Rng;
use winit::dpi::PhysicalSize;
use game_engine::engine::physics_engine::collision::collision_body::CollisionBody;
use game_engine::engine::{physics_engine::integrator::verlet::VerletIntegrator, run::run, Simulation};

struct FlappyBird {
    broadphase: Box<dyn BroadPhase>,
    narrowphase: Box<dyn NarrowPhase>,
    integrator: VerletIntegrator,
    dt: f32,
    window_size: Vector2<f32>,

    // Game specifics
    score: u32,
    next_pipe_pair_idx: usize,
    passed_pipe_pair_idx: usize,
}

const PIPE_START_X: f32 = 100.0;
const PIPE_WIDTH: f32 = 250.0;
const PIPE_PAIR_DISTANCE_Y: f32 = 400.0;
const PIPE_PAIR_DISTANCE_X: f32 = 1000.0;
const PIPE_PAIR_HOLE_WIEIGHT_RANGE_ABS: f32 = 0.75;
const PIPE_PAIR_VELOCITY_X: f32 = -4.0;

impl FlappyBird {
    fn new(window_size: &PhysicalSize<u32>) -> Self {
        let dt = 0.001;
        let gravity: f32 = 1000000.0;
        let accelleration = Vector3::new(0.0, -gravity, 0.0);
        let player_x = -1.0*((window_size.width / 2) as f32);
        let player_pos = Vector3::new(player_x, 0.0,0.0);
        let player = CollisionBody::circle(0, Vector3::zero(), accelleration, 
            player_pos.clone(), player_pos, 50.0, Vector3::new(255.0,0.0,0.0)); 
        
        let mut rng = rand::thread_rng();
        let r = PIPE_PAIR_HOLE_WIEIGHT_RANGE_ABS;
        let (top_pipe1, bot_pipe1) = Self::create_pipe_pair(1, 2, PIPE_START_X, PIPE_WIDTH,
            PIPE_PAIR_DISTANCE_Y, rng.gen_range(-r..r), window_size.height as f32, PIPE_PAIR_VELOCITY_X);

        let (top_pipe2, bot_pipe2) = Self::create_pipe_pair(3, 4, PIPE_START_X + 1.0*PIPE_PAIR_DISTANCE_X, 
            PIPE_WIDTH, PIPE_PAIR_DISTANCE_Y, rng.gen_range(-r..r), window_size.height as f32, PIPE_PAIR_VELOCITY_X);
        
        let (top_pipe3, bot_pipe3) = Self::create_pipe_pair(5, 6, PIPE_START_X + 2.0*PIPE_PAIR_DISTANCE_X, 
            PIPE_WIDTH, PIPE_PAIR_DISTANCE_Y, rng.gen_range(-r..r), window_size.height as f32, PIPE_PAIR_VELOCITY_X);

        let integrator = VerletIntegrator::new(f32::MAX,
            vec![player, top_pipe1, bot_pipe1, top_pipe2, bot_pipe2, top_pipe3, bot_pipe3]);

        let window_size = Vector2::new(window_size.width as f32, window_size.height as f32);
        
        let broadphase = Box::new(broadphase::blockmap::BlockMap::new(window_size.x));
        let narrowphase = Box::new(narrowphase::naive::Naive::new(IdentityCollisionSolver::new()));

        let next_pipe_pair_idx = 1;
        let passed_pipe_pair_idx = integrator.get_bodies().len() -1;
        let score = 0;

        Self {
            integrator, dt, window_size, next_pipe_pair_idx, passed_pipe_pair_idx, score,
            broadphase, narrowphase,
        }
    }

    fn create_pipe_pair(
        id_top: usize, id_bot: usize,
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
        let top = CollisionBody::rectangle(id_top, velocity, acceleration, top_prev_position,
            top_position, pipe_width, window_height*2.0, color);

        let bot_position = Vector3::new(x, -window_height, 0.0);
        let bot_prev_position = bot_position - velocity;
        let bot = CollisionBody::rectangle(id_bot, velocity, acceleration, bot_prev_position,
            bot_position, pipe_width,  (window_height - hole_size/2.0)*bot_weight, color) ;

        (top, bot)
    }

    fn is_player_passed_next_pipe_pair(
        bodies: &Vec<CollisionBody>, next_idx: usize 
    ) -> bool {
        let next_pipe_x = bodies[next_idx].position.x;
        let next_pipe_pair_right_edge = next_pipe_x + PIPE_WIDTH;
        let player_x = bodies[0].position.x;
        return next_pipe_pair_right_edge < player_x;
    }

    fn is_passed_pipe_pair_off_left_screen(
        bodies: &Vec<CollisionBody>, passed_idx: usize, window_width: f32
    ) -> bool {
        let passed_pipe_pair_right_edge = bodies[passed_idx].position.x + PIPE_WIDTH;
        let left_window_edge = -1.0*(window_width);
        return passed_pipe_pair_right_edge < left_window_edge;
    }

}

impl Simulation for FlappyBird {

    fn update(&mut self) {
        self.integrator.update(self.dt);
        
        let bodies = self.integrator.get_bodies_mut();
        let player = &bodies[0];
        if player.position.y.abs() > self.window_size.y {
            panic!("Game Over! Total score: {}", self.score);
        }
    
        let candidates = self.broadphase.collision_detection(bodies);
        
        if candidates.len() != 0 {
            println!("{:?}", candidates);
        }

        let graphs: Vec<CollisionGraph> = candidates.iter()
            .map(|cs| self.narrowphase.collision_detection(bodies,cs))
            .collect();

        let player_id: usize = 0;
        for g in graphs {
           let player_has_collided = g.collisions.iter()
               .fold(false, |acc, (i,j)| acc || i == &player_id || j == &player_id);
            if player_has_collided {
                panic!("Game Over! Total score: {}", self.score);
            }
        }

        if Self::is_passed_pipe_pair_off_left_screen(
            bodies, self.passed_pipe_pair_idx, self.window_size.x)
        {
            let current_x = bodies[self.passed_pipe_pair_idx].position.x;
            let largest_x = bodies.iter()
                              .fold(current_x, |acc,b| f32::max(acc, b.position.x));
            let new_x = largest_x + PIPE_PAIR_DISTANCE_X;
            
            // This is the lazy way of moving the pipes off the left screen to the right
            let mut rng = rand::thread_rng();
            let r = PIPE_PAIR_HOLE_WIEIGHT_RANGE_ABS;
            let old_top = &bodies[self.passed_pipe_pair_idx];
            let old_bot = &bodies[self.passed_pipe_pair_idx +1];
            let (new_top, new_bot) = Self::create_pipe_pair(
                old_top.id, old_bot.id, new_x, PIPE_WIDTH, PIPE_PAIR_DISTANCE_Y, 
                rng.gen_range(-r..r), self.window_size.y, PIPE_PAIR_VELOCITY_X);
            
            bodies[self.passed_pipe_pair_idx] = new_top;
            bodies[self.passed_pipe_pair_idx + 1] = new_bot;
        }
        
        if Self::is_player_passed_next_pipe_pair(bodies,self.next_pipe_pair_idx) { 
            self.passed_pipe_pair_idx = self.next_pipe_pair_idx;
            self.next_pipe_pair_idx = (self.next_pipe_pair_idx + 2) % bodies.len();
            if self.next_pipe_pair_idx == 0 {
                self.next_pipe_pair_idx += 1;
            }
            self.score += 1;
            println!("Score: {}", self.score);
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
    let window_size = PhysicalSize::new(1024,600);
    let flappy_bird = FlappyBird::new(&window_size);
    pollster::block_on(run(flappy_bird, window_size, 20));
}
