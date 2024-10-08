use std::time::{Duration, Instant};

use cgmath::{Vector2, Vector3, Zero};
use game_engine::engine::game_engine::GameEngineBuilder;
use game_engine::engine::physics_engine::broadphase::BroadPhase;
use game_engine::engine::physics_engine::collision::collision_candidates::CollisionCandidates;
use game_engine::engine::physics_engine::collision::collision_handler::IdentityCollisionSolver;
use game_engine::engine::physics_engine::narrowphase::NarrowPhase;
use game_engine::engine::physics_engine::{broadphase, narrowphase};
use game_engine::engine::renderer_engine::asset::asset::Asset;
use game_engine::engine::renderer_engine::asset::font::{Font, Writer};
use game_engine::engine::renderer_engine::asset::sprite_sheet::SpriteCoordinate;
use game_engine::engine::renderer_engine::post_process::PostProcessFilterId;
use game_engine::engine::renderer_engine::render_engine::RenderEngineControl;
use game_engine::engine::{PhysicsEngine, RenderEngine};
use game_state::GameState;
use rand::Rng;
use winit::dpi::PhysicalSize;
use game_engine::engine::physics_engine::collision::collision_body::CollisionBody;
use game_engine::engine::physics_engine::integrator::verlet::VerletIntegrator;

mod game_state;

struct FlappyBird {
    broadphase: Box<dyn BroadPhase>,
    narrowphase: Box<dyn NarrowPhase>,
    integrator: VerletIntegrator,
    dt: f32,
    window_size: Vector2<f32>,

    game_state: GameState,
    // Game specifics
    score: u32,
    next_pipe_pair_idx: usize,
    passed_pipe_pair_idx: usize,
    time_of_game_over: Instant,
    restart_available: bool,
    flash_restart_text: bool,
    flash_counter: Instant,
    
}

const PIPE_START_X: f32 = 200.0;
const PIPE_WIDTH: f32 = 350.0;
const PIPE_HEIGHT: f32 = 700.0;
const PIPE_PAIR_DISTANCE_Y: f32 = 430.0;
const PIPE_PAIR_DISTANCE_X: f32 = 1000.0;
const PIPE_PAIR_HOLE_WIEIGHT_RANGE_ABS: f32 = 0.5;
const PIPE_PAIR_VELOCITY_X: f32 = -4.0;

impl FlappyBird {
    fn new(window_size: &PhysicalSize<u32>) -> Self {
        let dt = 0.001;
        let gravity: f32 = 1000000.0;
        let accelleration = Vector3::new(0.0, -gravity, 0.0);
        let player_x = -1.0*((window_size.width / 2) as f32);
        let player_pos = Vector3::new(player_x, 0.0,0.0);
        let mut player = CollisionBody::circle(0, Vector3::zero(), accelleration, 
            player_pos.clone(), player_pos, 50.0, Vector3::new(255.0,0.0,0.0)); 
        
        player.set_sprite(SpriteCoordinate::new([2.0,0.0], [3.0,1.0]));

        let mut rng = rand::thread_rng();
        let r = PIPE_PAIR_HOLE_WIEIGHT_RANGE_ABS;
        let (top_pipe1, bot_pipe1) = Self::create_pipe_pair(1, 2, PIPE_START_X, [PIPE_WIDTH, PIPE_HEIGHT],
            PIPE_PAIR_DISTANCE_Y, rng.gen_range(-r..r), window_size.height as f32, PIPE_PAIR_VELOCITY_X);
        let (top_pipe2, bot_pipe2) = Self::create_pipe_pair(3, 4, PIPE_START_X + 1.0*PIPE_PAIR_DISTANCE_X, 
            [PIPE_WIDTH, PIPE_HEIGHT], PIPE_PAIR_DISTANCE_Y, rng.gen_range(-r..r), window_size.height as f32, PIPE_PAIR_VELOCITY_X);
        
        let (top_pipe3, bot_pipe3) = Self::create_pipe_pair(5, 6, PIPE_START_X + 2.0*PIPE_PAIR_DISTANCE_X, 
            [PIPE_WIDTH, PIPE_HEIGHT], PIPE_PAIR_DISTANCE_Y, rng.gen_range(-r..r), window_size.height as f32, PIPE_PAIR_VELOCITY_X);

        let integrator = VerletIntegrator::new(f32::MAX,
            vec![player, top_pipe1, bot_pipe1, top_pipe2, bot_pipe2, top_pipe3, bot_pipe3]);

        let window_size = Vector2::new(window_size.width as f32, window_size.height as f32);
        
        let broadphase = Box::new(broadphase::blockmap::BlockMap::new(window_size.x));
        let narrowphase = Box::new(narrowphase::naive::Naive::new(IdentityCollisionSolver::new()));

        let next_pipe_pair_idx = 1;
        let passed_pipe_pair_idx = integrator.get_bodies().len() -1;
        let score = 0;
        let game_state = GameState::Running;
        let time_of_game_over = Instant::now();
        let restart_available = false;
        let flash_restart_text = false;
        let flash_counter = Instant::now();
        Self {
            integrator, dt, window_size, next_pipe_pair_idx, passed_pipe_pair_idx, score,
            broadphase, narrowphase, game_state, time_of_game_over, flash_restart_text,
            flash_counter, restart_available
        }
    }

    fn restart_game(&mut self) {
        let gravity: f32 = 1000000.0;
        let accelleration = Vector3::new(0.0, -gravity, 0.0);
        let player_x = -1.0*(self.window_size.y / 2.0);
        let player_pos = Vector3::new(player_x, 0.0,0.0);
        let mut player = CollisionBody::circle(0, Vector3::zero(), accelleration, 
            player_pos.clone(), player_pos, 50.0, Vector3::new(255.0,0.0,0.0)); 
        
        player.set_sprite(SpriteCoordinate::new([2.0,0.0], [3.0,1.0]));

        let mut rng = rand::thread_rng();
        let r = PIPE_PAIR_HOLE_WIEIGHT_RANGE_ABS;
        let (top_pipe1, bot_pipe1) = Self::create_pipe_pair(1, 2, PIPE_START_X, [PIPE_WIDTH, PIPE_HEIGHT],
            PIPE_PAIR_DISTANCE_Y, rng.gen_range(-r..r), self.window_size.y, PIPE_PAIR_VELOCITY_X);
        let (top_pipe2, bot_pipe2) = Self::create_pipe_pair(3, 4, PIPE_START_X + 1.0*PIPE_PAIR_DISTANCE_X, 
            [PIPE_WIDTH, PIPE_HEIGHT], PIPE_PAIR_DISTANCE_Y, rng.gen_range(-r..r), self.window_size.y, PIPE_PAIR_VELOCITY_X);
        
        let (top_pipe3, bot_pipe3) = Self::create_pipe_pair(5, 6, PIPE_START_X + 2.0*PIPE_PAIR_DISTANCE_X, 
            [PIPE_WIDTH, PIPE_HEIGHT], PIPE_PAIR_DISTANCE_Y, rng.gen_range(-r..r), self.window_size.y, PIPE_PAIR_VELOCITY_X);
  
        self.integrator = VerletIntegrator::new(f32::MAX,
            vec![player, top_pipe1, bot_pipe1, top_pipe2, bot_pipe2, top_pipe3, bot_pipe3]);
        self.next_pipe_pair_idx = 1;
        self.passed_pipe_pair_idx = self.integrator.get_bodies().len() -1;
        self.score = 0;
        self.restart_available = false;
        self.flash_restart_text = false;
        self.game_state = GameState::Running;
    }



    fn create_pipe_pair(
        id_top: usize, id_bot: usize,
        x: f32, pipe_dims: [f32; 2], hole_size:f32, 
        hole_weight:f32, window_height:f32, velocity_x: f32
    ) -> (CollisionBody, CollisionBody) {

        let velocity = Vector3::new(velocity_x,0.0,0.0);
        let acceleration = Vector3::zero();
        let color = Vector3::new(0.0,255.0,0.0);

        let top_y = hole_size / 2.0;
        let bot_y = (-1.0 * hole_size / 2.0) - pipe_dims[1];

        let offset = window_height*hole_weight;
        let offset_top_y = top_y + offset;
        let offset_bot_y = bot_y + offset;

        let top_position = Vector3::new(x, offset_top_y,0.0);
        let top_prev_position = top_position - velocity;
        let mut top = CollisionBody::rectangle(id_top, velocity, acceleration, top_prev_position,
            top_position, pipe_dims[0], pipe_dims[1], color);
        top.set_sprite(SpriteCoordinate::new([1.0,0.0], [2.0, 2.0]));


        let bot_position = Vector3::new(x, offset_bot_y, 0.0);
        let bot_prev_position = bot_position - velocity;
        let mut bot = CollisionBody::rectangle(id_bot, velocity, acceleration, bot_prev_position,
            bot_position, pipe_dims[0],  pipe_dims[1], color) ;

        bot.set_sprite(SpriteCoordinate::new([0.0,0.0], [1.0, 2.0]));

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

impl PhysicsEngine for FlappyBird {

    fn update(&mut self) {
        
        match self.game_state {
            GameState::Running => (),
            GameState::GameOver => {
                if self.time_of_game_over.elapsed() > Duration::from_secs(3) {
                    self.restart_available = true;
                }
                return;
            }
            _ => return,
        }

        self.integrator.update(self.dt);
        
        let bodies = self.integrator.get_bodies_mut();
        let player = &bodies[0];
        if player.position.y.abs() > self.window_size.y {
            self.game_state = GameState::GameOver;
            self.time_of_game_over = Instant::now();
            return;
        }
    
        // We can unfortunately not use the developed broadphase collision detection,
        // BlockMap, because the size difference between the player and the pipe is 
        // too large. Instead we do a project specific broadphase.
        let player_id: usize = 0;
        let candidates = CollisionCandidates::new(
            vec![player_id, self.next_pipe_pair_idx, self.next_pipe_pair_idx +1]);
        let graph = self.narrowphase.collision_detection(bodies, &candidates);

        // Check if player has collidided with pipe
        let player_has_collided = graph.collisions.iter()
            .fold(false, |acc, (i,j)| acc || i == &player_id || j == &player_id);
        if player_has_collided {
            self.game_state = GameState::GameOver;
            self.time_of_game_over = Instant::now();
            return;
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
                old_top.id, old_bot.id, new_x, [PIPE_WIDTH, PIPE_HEIGHT], PIPE_PAIR_DISTANCE_Y, 
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
        }

    }

    fn get_bodies(&self) -> &Vec<CollisionBody> {
        &self.integrator.get_bodies()
    }

    fn jump(&mut self) {
        match self.game_state {
            GameState::Running => self.integrator.set_velocity_y(0, 20.0),
            GameState::GameOver => if self.restart_available {
                self.restart_game(); 
            },
            _ => (),
        }
    }
}

impl RenderEngine for FlappyBird {
    
    fn render(&mut self, engine_ctl: &mut RenderEngineControl) {
        let bodies = self.integrator.get_bodies();

        let mut texture_handle = engine_ctl.request_texture_handle();

        let rect_instances = game_engine::engine::util::get_rectangle_instances(bodies);
        let circle_instances = game_engine::engine::util::get_circle_instances(bodies);

        engine_ctl.render_background(&texture_handle).expect("Failed to render background.");
        engine_ctl.render_rectangles(&texture_handle, &rect_instances, false).expect("Failed to render rectangles");
        engine_ctl.render_circles(&texture_handle, &circle_instances, false).expect("Failed to render circles");

        let text_size = 110.;
        let score = format!("{}", self.score);
        let text = Writer::write(&score, &[0.0, 400.0, 0.0], text_size);
        engine_ctl.render_text(&texture_handle, text, false).expect("Failed to render score");


        match self.game_state {
            GameState::GameOver => {
                texture_handle = engine_ctl.run_post_process_filter(
                    &PostProcessFilterId::Tint, &texture_handle)
                    .expect("Failed to run post process filter tint");
                
                let text_size = 110.;
                let text = Writer::write("Game Over", &[-495.0, -55.0, 0.0], text_size);
                engine_ctl.render_text(&texture_handle, text, false).expect("Failed to render score");

                let text_size = 50.;
                let score_text = format!("You scored {} points", self.score);
                let text = Writer::write(&score_text, &[-(score_text.len() as f32 * text_size) / 2., -175.0, 0.0], text_size);
                engine_ctl.render_text(&texture_handle, text, false).expect("Failed to render score");

                if self.flash_restart_text && self.restart_available { 
                    let text_size = 50.;
                    let text = "Press space to restart" ;
                    let text_ = Writer::write(&text, &[-(text.len() as f32 * text_size) / 2., -325.0, 0.0], text_size);
                    engine_ctl.render_text(&texture_handle, text_, false).expect("Failed to render score");
                }

                if self.flash_restart_text && self.flash_counter.elapsed() > Duration::from_secs(2) {
                    self.flash_restart_text = !self.flash_restart_text;
                    self.flash_counter = Instant::now();
                }

                if !self.flash_restart_text && self.flash_counter.elapsed() > Duration::from_secs(1) {
                    self.flash_restart_text = !self.flash_restart_text;
                    self.flash_counter = Instant::now();
                }
            },
            _ => (),
        }

        engine_ctl.present(&texture_handle).expect("Failed to present texture");
    }
}

fn main() {
    let sprite_sheet_bytes = include_bytes!("../assets/sprite_sheet.png");
    let sprite_sheet_asset = Asset::sprite_sheet(sprite_sheet_bytes, 16,16);

    let background_bytes = include_bytes!("../assets/background.png");
    let background_asset = Asset::background(background_bytes);

    let font = Font::new(include_bytes!("../assets/font.png"), 11, 11);

    let window_size = PhysicalSize::new(1024,600);
    let flappy_bird = FlappyBird::new(&window_size);

    let game_engine = GameEngineBuilder::new()
        .window_title("Flappy Bird".to_string())
        .engine(flappy_bird)
        .font(font)
        .add_post_process_filters(&mut vec![PostProcessFilterId::Tint])
        .window_size(window_size)
        .sprite_sheet(sprite_sheet_asset)
        .background(background_asset)
        .target_frames_per_sec(60)
        .target_ticks_per_frame(1)
        .build();

    game_engine.run();
}
