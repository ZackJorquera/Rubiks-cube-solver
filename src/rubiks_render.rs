//! I built this using the OpenGL wrapper [`glium`].
//! 
//! [`glium`]: ../glium/index.html

use super::rubiks;

use std::time;
use glium::{glutin, Surface, Display, Program, Frame, self};

#[cfg(target_family = "unix")]
use nix::unistd::{fork, ForkResult};
#[cfg(target_family = "unix")]
use nix::sys::wait::waitpid;

/// `Vertex` is used for [`glium`]'s draw functions.
/// 
/// [`glium`]: ../glium/index.html
#[derive(Copy, Clone)]
struct Vertex 
{
    position: [f32; 2],
}
glium::implement_vertex!(Vertex, position);

#[derive(Copy, Clone)]
struct GridIndex
{
    rows: usize,
    cols: usize,
    index: (usize, usize)
}

pub struct RubikDrawer
{
    state: rubiks::RubiksCubeState,
}

impl RubikDrawer
{
    pub fn from_state(state: rubiks::RubiksCubeState) -> Self
    {
        RubikDrawer{state}
    }

    fn draw_quad(top_left: Vertex, top_right: Vertex, bottom_right: Vertex, bottom_left: Vertex,
        color: (f32,f32,f32), target: &mut Frame, display: &Display, program: &Program)
    {
        let shape = vec![top_left, top_right, bottom_right, bottom_left];

        // upload shape data to video memory
        let shape_vb = match glium::VertexBuffer::new(display, &shape)
        {
            Ok(vb) => vb,
            Err(glium::vertex::BufferCreationError::BufferCreationError(
                glium::buffer::BufferCreationError::OutOfMemory)) =>
                {
                    println!("{:?}", glium::buffer::BufferCreationError::OutOfMemory);
                    // I just want to skip for now
                    return;
                },
            e => e.unwrap() // I don't like this but the only other option is not supported err.
        };
        let indices = match glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &[0u16,1,3,1,2,3][..])
        {
            Ok(vb) => vb,
            Err(glium::index::BufferCreationError::BufferCreationError(
                glium::buffer::BufferCreationError::OutOfMemory)) =>
                {
                    println!("{:?}", glium::buffer::BufferCreationError::OutOfMemory);
                    // I just want to skip for now
                    return;
                },
            e => e.unwrap() // I don't like this but the only other option is not supported err.
        };
        
        let uniforms = glium::uniform! {
            rgb_color: color
        };

        target.draw(&shape_vb, &indices, program, &uniforms, &Default::default()).unwrap();
    }

    /// A wrapper around [`draw_quad`].
    /// 
    /// [`draw_quad`]: fn.draw_quad.html
    fn draw_square(grid_index: GridIndex, color: rubiks::Color, target: &mut Frame,
        display: &Display, program: &Program)
    {
        // Note, the glium draw space is from -1 to 1, how it should be

        let block_width = 2.0 / grid_index.cols as f32;
        let block_height = 2.0 / grid_index.rows as f32;

        let spacer_size = if grid_index.cols > 10 * 4 { 9.0 } else { 50.0 };

        let top_left = Vertex { position: [ 
                (grid_index.index.1 as f32 * block_width) - 1.0 + block_width / spacer_size, // last bit is a spacer
                -1.0 * ((grid_index.index.0 as f32 * block_height) - 1.0 + block_height / spacer_size) ]}; // TODO: are these flipped ?
        let bottom_right = Vertex { position: [ 
                ((grid_index.index.1+1) as f32 * block_width) - 1.0 - block_width / spacer_size,
                -1.0 * (((grid_index.index.0+1) as f32 * block_height) - 1.0 - block_height / spacer_size) ]}; // are these flipped ?;

        let top_right = Vertex { position: [ bottom_right.position[0],  top_left.position[1]] };
        let bottom_left = Vertex { position: [ top_left.position[0], bottom_right.position[1]] };

        let color_rgb = match color
        {
            rubiks::Color::White => (1.0, 1.0, 1.0),
            rubiks::Color::Green => (0.0, 1.0, 0.0),
            rubiks::Color::Red => (1.0, 0.0, 0.0),
            rubiks::Color::Blue => (0.0, 0.0, 1.0),
            rubiks::Color::Orange => (1.0, 0.5, 0.0),
            rubiks::Color::Yellow => (1.0, 1.0, 0.0)
        };

        Self::draw_quad(top_left, top_right, bottom_right, bottom_left, color_rgb, target, display, program)
    }

    fn draw_face(grid_index_top_left: GridIndex, grid_index_top_right: GridIndex, 
        target: &mut Frame, display: &Display, program: &Program)
    {
        // Note, the glium draw space is from -1 to 1, how it should be

        let block_width = 2.0 / grid_index_top_left.cols as f32;
        let block_height = 2.0 / grid_index_top_left.rows as f32;

        let top_left = Vertex { position: [ 
                (grid_index_top_left.index.1 as f32 * block_width) - 1.0, // last bit is a spacer
                -1.0 * ((grid_index_top_left.index.0 as f32 * block_height) - 1.0) ]}; // TODO: are these flipped ?
        let bottom_right = Vertex { position: [ 
                ((grid_index_top_right.index.1+1) as f32 * block_width) - 1.0,
                -1.0 * (((grid_index_top_right.index.0+1) as f32 * block_height) - 1.0) ]}; // are these flipped ?;

        let top_right = Vertex { position: [ bottom_right.position[0],  top_left.position[1]] };
        let bottom_left = Vertex { position: [ top_left.position[0], bottom_right.position[1]] };

        let color_rgb = (0.5, 0.5, 0.5);

        Self::draw_quad(top_left, top_right, bottom_right, bottom_left, color_rgb, target, display, program)
    }

    /// Renders a single frame for the game.
    fn draw_cube(cube_state: &rubiks::RubiksCubeState, display: &Display, program: &Program)
    {
        let mut target = display.draw();
        target.clear_color(1.0,1.0,1.0, 1.0);  // gray

        let cols = 4 * cube_state.size();
        let rows = 3 * cube_state.size();
        let n = cube_state.size();

        // UP
        Self::draw_face(GridIndex { cols, rows, index: (0,n) }, GridIndex { cols, rows, index: (n-1,2*n-1) }, &mut target, display, program);
        for i in 0..n
        {
            for j in 0..n
            {
                let grid_index = GridIndex { cols, rows, index: (i,j+n) };

                Self::draw_square(grid_index, cube_state.data_at(n*i + j), &mut target, display, program);
            }
        }

        // LFRB
        Self::draw_face(GridIndex { cols, rows, index: (n,0) }, GridIndex { cols, rows, index: (2*n-1,n-1) }, &mut target, display, program);
        Self::draw_face(GridIndex { cols, rows, index: (n,n) }, GridIndex { cols, rows, index: (2*n-1,2*n-1) }, &mut target, display, program);
        Self::draw_face(GridIndex { cols, rows, index: (n,2*n) }, GridIndex { cols, rows, index: (2*n-1,3*n-1) }, &mut target, display, program);
        Self::draw_face(GridIndex { cols, rows, index: (n,3*n) }, GridIndex { cols, rows, index: (2*n-1,4*n-1) }, &mut target, display, program);
        for i in 0..n
        {
            // Left
            for j in 0..n
            {
                let grid_index = GridIndex { cols, rows, index: (i+n,j) };
                Self::draw_square(grid_index, cube_state.data_at(n*n + n*i + j), &mut target, display, program);
            }
            
            // Front
            for j in 0..n
            {
                let grid_index = GridIndex { cols, rows, index: (i+n,j+n) };
                Self::draw_square(grid_index, cube_state.data_at(n*n*2 + n*i + j), &mut target, display, program);
            }
            
            // Right
            for j in 0..n
            {
                let grid_index = GridIndex { cols, rows, index: (i+n,j+2*n) };
                Self::draw_square(grid_index, cube_state.data_at(n*n*3 + n*i + j), &mut target, display, program);
            }
            
            // Back
            for j in 0..n
            {
                let grid_index = GridIndex { cols, rows, index: (i+n,j+3*n) };
                Self::draw_square(grid_index, cube_state.data_at(n*n*4 + n*i + j), &mut target, display, program);
            }
        }

        // Down
        Self::draw_face(GridIndex { cols, rows, index: (2*n,n) }, GridIndex { cols, rows, index: (3*n-1,2*n-1) }, &mut target, display, program);
        for i in 0..n
        {
            for j in 0..n
            {
                let grid_index = GridIndex { cols, rows, index: (i+2*n,j+n) };
                Self::draw_square(grid_index, cube_state.data_at(n*n*5 + n*i + j), &mut target, display, program);
            }
        }

        let _ = target.finish();
    }

    /// This is hacky, there must be a better way then to fork the process.
    #[cfg(target_family = "unix")]
    pub fn show(&self) -> ()
    {
        match unsafe{fork()} 
        {
            Ok(ForkResult::Parent { child, .. }) =>
            {
                match waitpid(child, None)
                {
                    Ok(_status) => (),//println!("{:?}", status),
                    Err(err) => println!("{:?}", err),
                };
            }
            Ok(ForkResult::Child) => 
            {
                let event_loop = glutin::event_loop::EventLoop::new();
                let wb = glutin::window::WindowBuilder::new()
                    .with_title("Rubik's Cube State");
                let cb = glutin::ContextBuilder::new().with_vsync(true);
                let display = glium::Display::new(wb, cb, &event_loop).unwrap();

                let vertex_shader_src = r#"
                    #version 140
                    in vec2 position;
                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                    }
                "#;

                let fragment_shader_src = r#"
                    #version 140
                    out vec4 color;
                    uniform vec3 rgb_color;
                    void main() {
                        color = vec4(rgb_color, 1.0);
                    }
                "#;

                let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

                let cube_state = self.state.clone();

                Self::draw_cube(&cube_state, &display, &program);

                event_loop.run(move |event, _, control_flow|
                {
                    // let frame_time = start.elapsed().as_secs_f32();
                    // start = time::Instant::now();
                    let next_frame_time = time::Instant::now() + time::Duration::from_millis(100); //time::Duration::from_nanos(33_333_333); // 60fps

                    *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

                    match event 
                    {
                        glutin::event::Event::WindowEvent { event, .. } => match event
                        {
                            glutin::event::WindowEvent::CloseRequested =>
                            {
                                *control_flow = glutin::event_loop::ControlFlow::Exit;
                                return;
                            },
                            glutin::event::WindowEvent::Resized(_) => Self::draw_cube(&cube_state, &display, &program),
                            _ => return,
                        },
                        _ => (),
                    }
                    
                    //Self::draw_cube(&cube_state, &display, &program);  // TODO: do we need the loop
                });
            },
            Err(_) => println!("Fork failed"),
        };
    }

    /// This is hacky, I don't know how to make it not end the process. 
    /// I mean i do, I have to use libc, but I don't want to
    #[cfg(target_family = "windows")]
    pub fn show(&self) -> !
    {
        println!("To use `show` that doesn't exit right after, use linux. Im too lazy to write good code. sorry.");

        let event_loop = glutin::event_loop::EventLoop::new();
        let wb = glutin::window::WindowBuilder::new()
            .with_title("Rubik's Cube State");
        let cb = glutin::ContextBuilder::new().with_vsync(true);
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        let vertex_shader_src = r#"
            #version 140
            in vec2 position;
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140
            out vec4 color;
            uniform vec3 rgb_color;
            void main() {
                color = vec4(rgb_color, 1.0);
            }
        "#;

        let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

        let cube_state = self.state.clone();

        Self::draw_cube(&cube_state, &display, &program);

        event_loop.run(move |event, _, control_flow|
        {
            // let frame_time = start.elapsed().as_secs_f32();
            // start = time::Instant::now();
            let next_frame_time = time::Instant::now() + time::Duration::from_millis(100); //time::Duration::from_nanos(33_333_333); // 60fps

            *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

            match event 
            {
                glutin::event::Event::WindowEvent { event, .. } => match event
                {
                    glutin::event::WindowEvent::CloseRequested =>
                    {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                        return;
                    },
                    glutin::event::WindowEvent::Resized(_) => Self::draw_cube(&cube_state, &display, &program),
                    _ => return,
                },
                _ => (),
            }
            
            //Self::draw_cube(&cube_state, &display, &program);  // TODO: do we need the loop
        })
    }
}
