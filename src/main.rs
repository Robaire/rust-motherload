#[macro_use]
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

pub mod gl_util;
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
            File::create("rust-motherload.log").unwrap(),
        ),
    ])
    .unwrap();

    let initial_window_size = 0.3;

    // Initialize SDL and create a window
    let (sdl_context, window, _gl_context, _video_subsystem) = {
        // Initialize SDL
        let sdl_context = match sdl2::init() {
            Ok(context) => context,
            Err(message) => panic!("SDL Init Failed: {}", message),
        };

        // Ask SDL to initialize the video system
        let video_subsystem = match sdl_context.video() {
            Ok(video_subsystem) => video_subsystem,
            Err(message) => panic!("Failed to create video subsystem: {}", message),
        };

        // Set the attributes of the OpenGL Context
        let gl_attributes = video_subsystem.gl_attr();
        gl_attributes.set_context_profile(GLProfile::Core);
        gl_attributes.set_context_flags().debug().set();
        gl_attributes.set_context_version(3, 3);

        // Determine the size of the window to open
        let (width, height) = match video_subsystem.desktop_display_mode(0) {
            Ok(display_mode) => {
                // Compute the width and height of the window
                let width = display_mode.w as f32 * initial_window_size;
                let height = width / (display_mode.w as f32 / display_mode.h as f32);

                (width as u32, height as u32)
            }
            Err(message) => panic!("Failed to get desktop display mode: {}", message),
        };

        // Create the window
        let window = match video_subsystem
            .window("Rust MotherLoad", width, height)
            .position_centered()
            .resizable()
            .opengl()
            .build()
        {
            Ok(window) => window,
            Err(message) => panic!("Failed to create window: {}", message),
        };

        // Create the OpenGL Context
        let gl_context = match window.gl_create_context() {
            Ok(context) => context,
            Err(message) => panic!("Failed to create OpenGL Context: {}", message),
        };

        // Load the OpenGL Functions
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::ffi::c_void);

        (sdl_context, window, gl_context, video_subsystem)
    };

    // Create the shaders and program objects
    let game_program = {
        let vertex =
            gl_util::shader::new_from_file("./src/shaders/game.vert", gl::VERTEX_SHADER).unwrap();
        let fragment =
            gl_util::shader::new_from_file("./src/shaders/game.frag", gl::FRAGMENT_SHADER).unwrap();

        let program = gl_util::program::create();

        gl_util::program::attach_shaders(program, vec![vertex, fragment]);
        gl_util::program::link(program).unwrap();

        program
    };

    let text_program = {
        let vertex =
            gl_util::shader::new_from_file("./src/shaders/text.vert", gl::VERTEX_SHADER).unwrap();
        let fragment =
            gl_util::shader::new_from_file("./src/shaders/text.frag", gl::FRAGMENT_SHADER).unwrap();

        let program = gl_util::program::create();

        gl_util::program::attach_shaders(program, vec![vertex, fragment]);
        gl_util::program::link(program).unwrap();

        program
    };

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

    enum OreType {
        Copper,
        Iron,
        Gold,
        Titanium,
    }

    enum TileType {
        Air,
        Regolith,
        Boulder,
        Treasure,
        Ore(OreType),
    }

    // Size of the world (x, y)
    let world_size = (10, 5);
    let score: Cell<u64> = Cell::new(0);
    let fuel: Cell<u64> = Cell::new(100);

    let tiles: RefCell<Vec<TileType>> =
        RefCell::new(Vec::with_capacity(world_size.0 * world_size.1));
    /*
     * Each tile is the same size, 1 square meter
     * Data to keep track of:
     *      - Tile type: air, dirt, rock, treasure, ore,
     *      - Position (but this can be determined by its place in the array?)
     *      - Point value (but that can be determined by its type)
     */

    let mut rng = SmallRng::from_entropy();
    for i in 0..(world_size.0 * world_size.1) {
        let val: f32 = rng.gen();

        if val < 0.1 {
            tiles.borrow_mut().push(TileType::Treasure);
        } else if val >= 0.1 && val < 0.5 {
            tiles.borrow_mut().push(TileType::Ore(OreType::Copper));
        } else if val >= 0.5 {
            tiles.borrow_mut().push(TileType::Regolith);
        }
    }

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

    // Physics
    let mut position: Cell<(f32, f32)> = Cell::new((0.0, 0.0)); // m

    let physics = || {};

    let actions = || {
        let c = commands.borrow();
        let mut p = position.get();

        if *c.get(&Command::Right).unwrap() {
            p.0 += 1.0;
        }

        if *c.get(&Command::Left).unwrap() {
            p.0 -= 1.0;
        }

        if *c.get(&Command::Up).unwrap() {
            p.1 -= 1.0;
        }

        if *c.get(&Command::Down).unwrap() {
            p.1 += 1.0;
        }

        if p.0 as usize >= world_size.0 - 1 {
            p.0 = (world_size.0 - 1) as f32;
        }

        if p.0 < 0.0 {
            p.0 = 0.0;
        }

        if p.1 as usize >= world_size.1 - 1 {
            p.1 = (world_size.1 - 1) as f32;
        }

        if p.1 < 0.0 {
            p.1 = 0.0;
        }

        if p != position.get() {
            fuel.set(fuel.get() - 10);
        }

        position.set(p);
    };

    let update = || {
        let mut t = tiles.borrow_mut();
        let p = position.get();
        let tile_index = p.1 as usize * world_size.0 + p.0 as usize;
        let mut s = score.get();

        // If this happens something went wrong
        if t.get(tile_index).is_none() {
            panic!("Tile index is none existent!");
        }

        // Update the score
        s += match t.get(tile_index).unwrap() {
            TileType::Regolith => 10,
            TileType::Treasure => 500,
            TileType::Ore(ore_type) => match ore_type {
                OreType::Copper => 15,
                OreType::Iron => 25,
                OreType::Gold => 50,
                OreType::Titanium => 100,
                _ => 0,
            },
            _ => 0,
        };

        // Update the score
        score.set(s);

        // Set the tile to air
        t[tile_index] = TileType::Air;

        if fuel.get() == 0 {
            println!("You ran out of fuel!");
            std::process::exit(0);
        }
    };

    let print_world = || {
        println!("Score: {}, Fuel: {}", score.get(), fuel.get());
        let position = position.get();
        let tiles = tiles.borrow();

        for y in 0..world_size.1 {
            for x in 0..world_size.0 {
                if y == position.1 as usize && x == position.0 as usize {
                    print!("M");
                } else {
                    match &tiles[y * world_size.0 + x] {
                        TileType::Air => print!("."),
                        TileType::Regolith => print!("#"),
                        TileType::Boulder => print!("B"),
                        TileType::Ore(_) => print!("O"),
                        TileType::Treasure => print!("T"),
                    }
                }
            }
            println!();
        }
        println!();
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

        // println!("Delta T: {}", delta_time);
        // println!("Cycles / Second: {}", 1.0 / delta_time);

        physics();
        actions();
        update();
        print_world();

        // Draw the new state to the screen
        gl_util::clear();

        // Swap the buffers
        window.gl_swap_window(); // This might block in order to synchronize with the display refresh rate!!!!

        let sleep_time = std::time::Duration::from_millis(5);
        std::thread::sleep(sleep_time);
    }
}
