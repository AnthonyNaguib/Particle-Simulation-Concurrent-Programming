#[macro_use]
extern crate glium;
use std::{sync::{Arc, atomic::{AtomicIsize, Ordering}}, time::SystemTime};
use rand::Rng;

const NUM_OF_THREADS: usize = 10;
const VALUES_PER_THREAD: usize = 100;
const NUM_OF_PARTICLES: usize = 10000;

const COOL_FACTOR: f32 = 2.0;
const INITIAL_SPEED:f32 = 0.009;

impl PartialEq for Particle {
    fn eq(&self, other: &Self) -> bool {
      self.x == other.x && self.y == other.y && self.z == other.z
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Particle{
    x: f32,
    y: f32,
    z: f32,
    x_vel: f32,
    y_vel: f32,
    z_vel: f32,
    mass: f32,
    temperature: f32
}
pub struct ParticleSystem{
    particles:Vec<Particle>,
    collision_counter: Arc<AtomicIsize>,
    floor_counter: Arc<AtomicIsize>,
}
impl Particle{
    pub fn new(x_param: f32, y_param: f32, z_param: f32, x_velocity: f32, y_velocity: f32, z_velocity: f32, mass_param: f32, temp_param: f32) -> Particle{
        Particle { x: x_param, y: y_param, z: z_param, x_vel: x_velocity, y_vel: y_velocity, z_vel: z_velocity, mass: mass_param, temperature: temp_param}
    }
}
impl ParticleSystem{
    pub fn new()-> ParticleSystem{
        ParticleSystem{particles: Vec::new(), 
        collision_counter: Arc::new(AtomicIsize::new(0)),
        floor_counter: Arc::new(AtomicIsize::new(0))}
    }
    pub fn increment(&mut self, delta_t: f32){
        let mut pool = scoped_threadpool::Pool::new(NUM_OF_THREADS as u32);

        pool.scoped(|scope| {
            for slice in self.particles.chunks_mut(VALUES_PER_THREAD){
                let arc_clone = self.floor_counter.clone();
                scope.execute(move || {
                    thread_main_increment(slice);
                    thread_main_wall_collision(slice, &arc_clone);
                    thread_main_temp_change(slice, delta_t);
                });
            }
        });
    }
    fn thread_main_particle_collision(&mut self){
        let mut particle_iter = self.particles.clone();
        let mut particle_clone = self.particles.clone();
    
        for particle_current in self.particles.iter_mut(){
    
            if particle_current.y < 0.5
            {
                for current_p in particle_iter.iter_mut(){
                    
                
                    if !(particle_current == current_p)
                    {
                        let dist_sq = (particle_current.x - current_p.x)*(particle_current.x - current_p.x) 
                                         + (particle_current.y - current_p.y)*(particle_current.y - current_p.y)
                                         + (particle_current.z - current_p.z)*(particle_current.z - current_p.z);
                        let rad_sq = (0.0125+0.0125)*(0.0125+0.0125);

                        if dist_sq < rad_sq{

                            self.collision_counter.fetch_add(1, Ordering::Relaxed);

                            let idx = particle_clone.iter().position(|particle| particle == current_p);

                            if let Some(idx) = idx {
                                if let Some(particle) = particle_clone.iter_mut().find(|particle| *particle == particle_current) {

                                    particle.mass += current_p.mass;
    
                                    particle.x_vel = (particle_current.mass * particle_current.x_vel + current_p.mass * current_p.x_vel)/(particle_current.mass + current_p.mass);
                                    particle.y_vel = (particle_current.mass * particle_current.y_vel + current_p.mass * current_p.y_vel)/(particle_current.mass + current_p.mass);
                                    particle.z_vel = (particle_current.mass * particle_current.z_vel + current_p.mass * current_p.z_vel)/(particle_current.mass + current_p.mass);

                                }
                                particle_clone.swap_remove(idx);
                            }
                        }
                    }
                }
            }
        }
        self.particles = particle_clone;
    }
}
fn thread_main_increment(slice :&mut [Particle]){
    for id in slice{

        let weight: f32 = id.mass * 9.8;
        let accel = 0.02 * weight * 9.8;

        id.y_vel += 0.02 * accel;

        id.x += id.x_vel * 2.0;
        id.y -= id.y_vel * 0.02;
        id.z += id.z_vel * 2.0;

        id.mass += rand::thread_rng().gen_range(0.001..0.1);
    }
}
fn thread_main_temp_change(slice :&mut [Particle], delta_t: f32){
    for id in slice{
        id.temperature = id.temperature - delta_t * COOL_FACTOR / id.mass;
    }
}
fn thread_main_wall_collision(slice :&mut [Particle], atomic_arc: &Arc<AtomicIsize>){
    for id in slice{
        if id.x < -1.0 || id.x > 1.0
        {
            id.x_vel = -(id.x_vel);
        }
        if id.z < -1.0 || id.z > 1.0
        {
            id.z_vel = -(id.z_vel);
        }
        if id.y < -1.0
        {
            atomic_arc.fetch_add(1, Ordering::Relaxed);

            id.x = rand::thread_rng().gen_range(-0.05..0.05);
            id.y = 1.0;
            id.z = rand::thread_rng().gen_range(-0.05..0.05);

            let shower_angle: f32 = rand::thread_rng().gen_range(-30.0..30.0);
            id.x_vel = INITIAL_SPEED * shower_angle.cos();
            id.y_vel = 0.0;
            id.z_vel = INITIAL_SPEED * shower_angle.sin();

            id.mass = 1.0;
            id.temperature = 1.0;
        }
    }
}fn main() {
    let mut ps = ParticleSystem::new();
    for _id in 0..NUM_OF_PARTICLES{
        let x: f32 = rand::thread_rng().gen_range(-0.05..0.05);
        let y: f32 = 1.0;
        let z: f32 = rand::thread_rng().gen_range(-0.05..0.05);

        let shower_angle: f32 = rand::thread_rng().gen_range(-30.0..30.0);
        let x_vel: f32 = INITIAL_SPEED * shower_angle.cos();
        let y_vel: f32 = 0.0;
        let z_vel: f32 = INITIAL_SPEED * shower_angle.sin();

        let mass: f32 = 1.0;
        let temp: f32 = 1.0;
        
        let particle = Particle::new(x, y, z, x_vel, y_vel, z_vel, mass, temp);
        ps.particles.push(particle)
    }

    #[allow(unused_imports)]
    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 3],
    }

    implement_vertex!(Vertex, position);

    let mut shape: Vec<Vertex> = Vec::new();
    let mut theta: f32 = 6.28319;
    while theta > 0.0{
        let x: f32 = 0.0 + 0.0125 * theta.sin();
        let y: f32 = 0.0 + 0.0125 * theta.cos();
        let z: f32 = 0.0 + 0.0125 * theta.sin();
        let vertex = Vertex { position: [x, y, z]};
        shape.push(vertex);
        theta -= 0.0174533;
    }

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

    let vertex_shader_src = r#"
        #version 140

        in vec3 position;

        uniform mat4 matrix;
        uniform mat4 perspective;  

        void main() {
            gl_Position = perspective * matrix * vec4(position, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        uniform vec4 color;

        out vec4 out_color;

        void main() {
            out_color = color;
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let mut delta_t = SystemTime::now();
    let mut temp_or_mass = true;
    let mut performance_mode = true;
    let mut collisions = false;

    event_loop.run(move |event, _, control_flow| {
        let start = SystemTime::now();
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                glutin::event::WindowEvent::KeyboardInput { device_id: _, input, is_synthetic: _} => {
                    if input.state ==  glutin::event::ElementState::Pressed {
                        println! ("Keypressed {:?}", input.virtual_keycode.unwrap());
                        let key = input.virtual_keycode.unwrap();
                        if  key == glium::glutin::event::VirtualKeyCode::G {
                            println! ("G key - Switching Colour Representation");
                            temp_or_mass = !temp_or_mass;
                        }
                        if  key == glium::glutin::event::VirtualKeyCode::F {
                            println! ("F key - Switching Mode");
                            performance_mode = !performance_mode;
                        }
                        if  key == glium::glutin::event::VirtualKeyCode::H {
                            println! ("H key - Collisions");
                            collisions = !collisions;
                        }
                    }
                },
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        ps.increment(delta_t.elapsed().unwrap().as_secs_f32());
        delta_t = SystemTime::now();

        if collisions{
            ps.thread_main_particle_collision();
        }
        let mut every_n_particle = 1;
        if performance_mode && ps.particles.len() >= 100000
        {
            every_n_particle = 10000;
        }
        else if performance_mode && ps.particles.len() >= 10000{
            every_n_particle = 1000;
        }
        else {
            every_n_particle = ps.particles.len();

        }

        // Get a drawing canvas
        let mut target = display.draw();

        // Clear the screen
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        // Calculate the data for the camera configuration
        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;
        let fov: f32 = 3.141592 / 2.0; // Field of view
        let zfar = 1024.0;  // Far clipping plain
        let znear = 0.1; // Near clipping plain
        let f = 1.0 / (fov / 2.0).tan();
        
        // Loop through each of 10 triangles
        for i in 0 .. every_n_particle {

            // Calculate the x and y position of a triangle
            let pos_x : f32 = ps.particles[i].x;
            let pos_y : f32 = ps.particles[i].y;
            let pos_z : f32 = ps.particles[i].z;

            // Calculate the colours
            let mut red = 0.0;
            let mut green = 0.0;
            let mut blue = 0.0;


            if temp_or_mass{
                red = ps.particles[i].temperature;
                green = 0.0;
                blue = 1.0 - ps.particles[i].temperature;
            }
            else {
                red = ps.particles[i].mass*0.25;
                green = (1.0 - ps.particles[i].mass*0.15).abs();
                blue = ps.particles[i].mass;
            }

            // Create a 4x4 matrix to hold the position and orientation of the triangle
            // and a 4x4 matrix to hold the camera perspective correction
            let uniforms = uniform! {
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [pos_x, pos_y, pos_z, 1.0],
                ],
                perspective: [
                    [f*aspect_ratio, 0.0, 0.0, 0.0],
                    [0.0, f, 0.0, 0.0],
                    [0.0, 0.0, (zfar+znear)/(zfar-znear), 1.0],
                    [0.0, 0.0, -(2.0*zfar*znear)/(zfar-znear), 2.25],
                ],
                color: [red, green, blue, 1.0 as f32]
            };

            // Draw the triangle
            target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        }

        // Display the new image
        target.finish().unwrap();

        let count = start.elapsed().unwrap().as_micros();
        //println!("Count: {}", count);
        println!("Collision Count: {:?}, Floor Count: {:?}", ps.collision_counter, ps.floor_counter);
        // End render loop
    });
}
