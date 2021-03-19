extern crate gl;

extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

use std::collections::HashMap;
use std::time::Instant;

use rand::prelude::*;
use std::cell::{Cell, RefCell};

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

fn main() {
    let initial_window_size = 0.5;

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

    // Size of the world (x, y)
    let world_size = (10, 5);
    let score: Cell<u64> = Cell::new(0);

    let tiles: RefCell<Vec<TileType>> =
        RefCell::new(Vec::with_capacity(world_size.0 * world_size.1));
    /*
     * Each tile is the same size, 1 square meter
     * Data to keep track of:
     *      - Tile type: air, dirt, rock, treasure, ore,
     *      - Position (but this can be determined by its place in the array?)
     *      - Point value (but that can be determined by its type)
     */

    for i in 0..(world_size.0 * world_size.1) {
        tiles.borrow_mut().push(TileType::Regolith);
        // if rand::random() {
        //     tiles.push(TileType::Regolith);
        // } else {
        //     tiles.push(TileType::Air);
        // }
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

    let physics = || {
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

        position.set(p);
    };

    let update_board = || {
        let mut t = tiles.borrow_mut();
        let p = position.get();

        if t.get(p.1 as usize * world_size.0 + p.0 as usize).is_some() {
            t[p.1 as usize * world_size.0 + p.0 as usize] = TileType::Air;
        }
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
        update_board();
        print_world(world_size, position.get(), &tiles.borrow());

        // Draw the new state to the screen
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Swap the buffers
        window.gl_swap_window(); // This might wait in order to synchronize with the display refresh rate!!!!

        let sleep_time = std::time::Duration::from_millis(5);
        std::thread::sleep(sleep_time);
    }
}

fn print_world(world_size: (usize, usize), position: (f32, f32), tiles: &Vec<TileType>) {
    println!("World:");

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
}
