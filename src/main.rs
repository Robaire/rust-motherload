extern crate log;
extern crate simplelog;
use simplelog::*;
use std::fs::File;

extern crate gl;

extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

use std::collections::HashMap;
use std::time::Instant;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::cell::{Cell, RefCell};

extern crate gl_util;

pub mod map;
pub mod text;

fn main() {
    // Setup logging
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("rust-motherload.log").expect("Could not create log file"),
        ),
    ])
    .unwrap();

    let initial_window_size = 0.3;

    // Initialize SDL and create a window
    let (sdl_context, window, _gl_context, _video_subsystem) = {
        // Initialize SDL
        let sdl_context = sdl2::init().expect("SDL Init Failed");

        // Ask SDL to initialize the video system
        let video_subsystem = sdl_context
            .video()
            .expect("Failed to create video subsystem");

        // Set the attributes of the OpenGL Context
        let gl_attributes = video_subsystem.gl_attr();
        gl_attributes.set_context_profile(GLProfile::Core);
        gl_attributes.set_context_flags().debug().set();
        gl_attributes.set_context_version(3, 3);

        // Determine the size of the window to open
        let (width, height) = {
            let display_mode = video_subsystem
                .desktop_display_mode(0)
                .expect("Failed to get desktop display mode");
            let width = display_mode.w as f32 * initial_window_size;
            let height = width / (display_mode.w as f32 / display_mode.h as f32);
            (width as u32, height as u32)
        };

        // Create the window
        let window = video_subsystem
            .window("Rust MotherLoad", width, height)
            .position_centered()
            .resizable()
            .opengl()
            .build()
            .expect("Failed to create window");

        // Create the OpenGL Context
        let gl_context = window
            .gl_create_context()
            .expect("Failed to create OpenGL Context");

        // Load the OpenGL Functions
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::ffi::c_void);

        (sdl_context, window, gl_context, video_subsystem)
    };

    // TODO: Build up text rendering system for HUD
    // TODO: Refactor control input code into its own module

    #[derive(Eq, PartialEq, Hash)]
    enum Command {
        Exit,
        Right,
        Left,
        Up,
        Down,
        Interact,
    }

    let commands: RefCell<HashMap<Command, bool>> = RefCell::new(HashMap::new());
    {
        let mut c = commands.borrow_mut();
        c.insert(Command::Exit, false);
        c.insert(Command::Right, false);
        c.insert(Command::Left, false);
        c.insert(Command::Up, false);
        c.insert(Command::Down, false);
        c.insert(Command::Interact, false);
        c.shrink_to_fit();
    }

    let mut inputs: HashMap<Keycode, Command> = HashMap::new();
    inputs.insert(Keycode::E, Command::Exit);
    inputs.insert(Keycode::Escape, Command::Exit);
    inputs.insert(Keycode::W, Command::Up);
    inputs.insert(Keycode::Up, Command::Up);
    inputs.insert(Keycode::S, Command::Down);
    inputs.insert(Keycode::Down, Command::Down);
    inputs.insert(Keycode::A, Command::Left);
    inputs.insert(Keycode::Left, Command::Left);
    inputs.insert(Keycode::D, Command::Right);
    inputs.insert(Keycode::Right, Command::Right);
    inputs.insert(Keycode::Space, Command::Interact);
    inputs.shrink_to_fit();

    // Load the tile rendering shaders
    let tile_program = {

        // Create shader objects
        let vertex_id = gl_util::shader::new_from_file("./src/shaders/tiles.vert", gl::VERTEX_SHADER).unwrap();
        let frag_id = gl_util::shader::new_from_file("./src/shaders/tiles.frag", gl::FRAGMENT_SHADER).unwrap();

        // Create a shader program
        let program_id = gl_util::program::create();
        gl_util::program::attach_shaders(program_id, vec![vertex_id, frag_id]);
        gl_util::program::link(program_id).unwrap();

        program_id
    };

    // Create a world map
    let map = map::Map::generate(30, 10);

    // Generate all the verticies of the world grid
    // Each tile needs to know which world grid verticies make up its corners

    // Size of the world (x, y)
    let world_size = (30, 10);
    let score: Cell<u64> = Cell::new(0);
    let fuel: Cell<u64> = Cell::new(100);

    /*
     * Each tile is the same size, 1 square meter
     * Data to keep track of:
     *      - Tile type: air, dirt, rock, treasure, ore,
     *      - Position (but this can be determined by its place in the array?)
     *      - Point value (but that can be determined by its type)
     */

    // Time
    let mut now = Instant::now();
    let mut then = now;
    let delta_time: Cell<f64> = Cell::new(0.0);

    // Calculates Delta-Time
    let mut tick = || {
        now = std::time::Instant::now();
        delta_time.set((now - then).as_secs_f64());
        then = now;
    };

    // Enter the main event loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    'main_loop: loop {
        // Clear the event queue
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown { keycode, .. } => match inputs.get(&keycode.unwrap()) {
                    Some(command) => match commands.borrow_mut().get_mut(&command) {
                        Some(state) => *state = true,
                        None => {}
                    },
                    None => {
                        println!("The {} key does not do anything!", keycode.unwrap());
                    }
                },
                Event::KeyUp { keycode, .. } => match inputs.get(&keycode.unwrap()) {
                    Some(command) => match commands.borrow_mut().get_mut(&command) {
                        Some(state) => *state = false,
                        None => {}
                    },
                    None => {}
                },
                _ => {}
            }
        }

        // Calculate the delta time
        tick();

        println!("Delta T: {} s", delta_time.get());
        println!("Frequency: {} Hz", 1.0 / delta_time.get());

        // physics();
        // actions();
        // update();
        // map.print();

        // draw_world();
        // Draw World
        // Draw Player
        // Draw HUD / Menus

        // Draw the new state to the screen
        gl_util::clear();

        // Swap the buffers
        window.gl_swap_window(); // This blocks to sychronize with the display refresh rate

        // let sleep_time = std::time::Duration::from_millis(5);
        // std::thread::sleep(sleep_time);
    }
}
