use game_engine::engine::{physics_engine::{collision::CollisionBody, integrator::verlet::VerletIntegrator}, renderer_engine::{shapes::circle::Circle, vertex::Vertex}, Simulation};
use cgmath::{Vector3, Zero};
use winit::dpi::PhysicalSize;
use game_engine::engine::run::run;

struct FlappyBird {
    player: CollisionBody,
    integrator: VerletIntegrator,

    // Render data
    colors: Vec<Vector3<f32>>,
    indices: Vec<u16>,
    vertices: Vec<Vertex>,
    num_indices: u32,
}

impl Simulation for FlappyBird {

    fn new(_window_size: &PhysicalSize<u32>) -> Self {
        let player = CollisionBody::new(0, Vector3::zero(), Vector3::zero(), Vector3::zero(), 50.0); 
  
        let integrator = VerletIntegrator::new(f32::MAX, vec![Vector3::zero()], vec![player]);
        let colors = vec![Vector3::new(255.0,0.0,0.0)];

        let vertices = Circle::compute_vertices([0.0,0.0,0.0], 1.0);
        let indices = Circle::compute_indices();
        let num_indices = Circle::get_num_indices();

        Self {
            player, integrator,
            colors, indices, vertices, num_indices
        }
    }

    fn update(&mut self) {
    }


    fn get_bodies(&self) -> &Vec<game_engine::engine::physics_engine::collision::CollisionBody> {
        &self.integrator.get_bodies()
    }

    fn get_vertices(&self) -> &Vec<game_engine::engine::renderer_engine::vertex::Vertex> {
        &self.vertices
    }

    fn get_indices(&self) -> &Vec<u16> {
        &self.indices
    }
    
    fn get_colors(&self) -> &Vec<Vector3<f32>> {
        &self.colors
    }

    fn get_num_active_instances(&self) -> u32 {
        1
    }

    fn get_target_num_instances(&self) -> u32 {
        1
    }

    fn get_num_indices(&self) -> u32 {
        self.num_indices
    }

    fn log_performance(&mut self) {
        todo!();
    }
}

fn main() {
    let window_size = PhysicalSize::new(600,600);
    let mut flappy_bird = FlappyBird::new(&window_size);
    pollster::block_on(run(&mut flappy_bird, window_size));
}
