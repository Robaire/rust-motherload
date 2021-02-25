extern crate gl;

extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode::*;
use sdl2::video::GLProfile;

use std::time::{Duration, Instant};

fn main() {
    let initial_window_size = 0.5;

    // Initialize SDL and create a window
    let (sdl_context, window, _gl_context, video_subsystem) = {
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

    let mut position = (0.0, 0.0);
    let mut velocity = (0.0, 0.0);
    let mut acceleration = (0.0, 0.0);

    let max_velocity = (1.0, 1.0);

    let mut then = std::time::Instant::now();

    // Enter the main event loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    'main_loop: loop {

        // Clear the event queue
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown { keycode, .. } => match keycode.unwrap() {
                    W => velocity.1 = 1.0,
                    A => velocity.0 = -1.0,
                    S => velocity.1 = -1.0,
                    D => velocity.0 = 1.0,
                    _ => {}
                },
                Event::KeyUp { keycode, .. } => match keycode.unwrap() {
                    W | S => velocity.1 = 0.0,
                    A | D => velocity.0 = 0.0,
                    _ => {}
                },
                _ => {}
            };
        }

        // Calculate the delta time
        let now = Instant::now();
        let delta_time = (now - then).as_secs_f64();
        then = now;

        println!("Delta T: {}", delta_time);
        println!("Cycles / Second: {}", 1.0 / delta_time);

        // Integrate new state given input
        position.0 += velocity.0;
        position.1 += velocity.1;

        println!("({}, {})", position.0, position.1);

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
