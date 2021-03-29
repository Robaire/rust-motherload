use std::ffi::CString;

extern crate gl;
use gl::types::{GLenum};

extern crate image;

/// Get error
pub fn get_error() -> GLenum {
    return unsafe { gl::GetError() };
}

/// Clear
pub fn clear() {
    unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
}

pub mod shader {

    use gl::types::{GLchar, GLenum};
    use std::ffi::CString;

    /// Create a shader using a string as the source code
    /// # Arguments
    /// `source` - The shader source code string
    /// `kind` - The kind of shader to create
    pub fn new_from_string(source: String, kind: GLenum) -> Result<u32, String> {
        // Create a shader
        let shader_id = create(kind)?;

        // Compile the shader
        compile(shader_id, source);

        // Check if the shader compiled
        let status = get_parameter(shader_id, gl::COMPILE_STATUS);

        if status == 1 {
            return Ok(shader_id);
        } else {
            return Err(get_info_log(shader_id));
        }
    }

    /// Create a shader using a file as the source code
    /// # Arguments
    /// `path` - Path to the source code file
    /// `kind` - The kind of shader to create
    pub fn new_from_file(path: &str, kind: GLenum) -> Result<u32, String> {
        // Read the source file in as a string
        match std::fs::read_to_string(path) {
            Ok(source) => new_from_string(source, kind),
            Err(message) => Err(message.to_string()),
        }
    }

    /// Creates a shader object on the GPU
    /// # Arguments
    /// * `kind` - The kind of shader to generate
    fn create(kind: GLenum) -> Result<u32, String> {
        let mut id = 0;
        unsafe {
            gl::CreateShader(kind);
        };

        if id == 0 {
            return Err("Shader could not be created".to_string());
        } else {
            return Ok(id);
        }
    }

    /// Compiles a shader program
    /// # Arguments
    /// * `id` - Shader Program ID
    /// * `source` - Shader program source code
    fn compile(id: u32, source: String) {
        unsafe {
            gl::ShaderSource(
                id,
                1,
                &CString::new(source).unwrap().as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(id);
        };
    }

    /// Get a shader parameter
    /// # Arguments
    /// `id` - Shader ID
    /// `param` - The shader parameter to retrieve
    fn get_parameter(id: u32, param: GLenum) -> i32 {
        let mut status: i32 = 0;
        unsafe {
            gl::GetShaderiv(id, param, &mut status);
        };

        status
    }

    /// Get a shader's info log
    /// `id` - Shader ID
    fn get_info_log(id: u32) -> String {
        let log_length = get_parameter(id, gl::INFO_LOG_LENGTH);

        let log: CString = {
            let mut buffer: Vec<u8> = Vec::with_capacity(log_length as usize + 1);
            buffer.extend([b' '].iter().cycle().take(log_length as usize));
            unsafe { CString::from_vec_unchecked(buffer) }
        };

        unsafe {
            gl::GetShaderInfoLog(
                id,
                log_length,
                std::ptr::null_mut(),
                log.as_ptr() as *mut GLchar,
            );
        };

        log.to_string_lossy().into_owned()
    }
}

pub mod program {
    use gl::types::{GLchar, GLenum};
    use std::ffi::CString;

    /// Set a shader program as used
    /// # Arugments
    /// * `id` - Shader Program ID
    pub fn set_used(id: u32) {
        unsafe {
            gl::UseProgram(id);
        }
    }

    /// Create a program object on the GPU
    fn create() -> u32 {
        unsafe { gl::CreateProgram() }
    }

    /// Attach a shader to a program object
    /// # Arguments
    /// `program` - The id of the program to attach the shader to
    /// `shaders` - The ids of the shaders to attach to the program
    pub fn attach_shaders(program: u32, shaders: Vec<u32>) {
        for shader in shaders {
            unsafe { gl::AttachShader(program, shader) };
        }
    }

    /// Detach shaders from a program
    /// # Arguments
    /// `program` - id of the program to attach the shader to
    /// `shaders` - ids of the shaders to attach to the program
    pub fn detach_shaders(program: u32, shaders: Vec<u32>) {
        for shader in shaders {
            unsafe { gl::DetachShader(program, shader) };
        }
    }

    /// Link a program
    /// # Arguments
    /// `program` - id of the program to link
    pub fn link(program: u32) -> Result<(), String> {
        unsafe {
            gl::LinkProgram(program);
        };

        let link_status = get_parameter(program, gl::LINK_STATUS);

        if link_status == 1 {
            Ok(())
        } else {
            Err(get_info_log(program))
        }
    }

    /// Deletes a program object
    /// # Arguments
    /// `program` - id of the program to delete
    pub fn delete(program: u32) {
        unsafe { gl::DeleteProgram(program) };
    }

    /// Get a program parameter
    /// # Arguments
    /// `id` - id of the program
    /// `param` - the program parameter to retrieve
    fn get_parameter(id: u32, param: GLenum) -> i32 {
        let mut status: i32 = 0;
        unsafe {
            gl::GetProgramiv(id, param, &mut status);
        };

        status
    }

    /// Get a programs's info log
    /// `id` - id of the program
    fn get_info_log(id: u32) -> String {
        let log_length = get_parameter(id, gl::INFO_LOG_LENGTH);

        let log: CString = {
            let mut buffer: Vec<u8> = Vec::with_capacity(log_length as usize + 1);
            buffer.extend([b' '].iter().cycle().take(log_length as usize));
            unsafe { CString::from_vec_unchecked(buffer) }
        };

        unsafe {
            gl::GetProgramInfoLog(
                id,
                log_length,
                std::ptr::null_mut(),
                log.as_ptr() as *mut GLchar,
            );
        };

        log.to_string_lossy().into_owned()
    }
}

/// Generates a buffer on the GPU and returns its id
pub fn generate_buffer() -> u32 {
    let mut id = 0;

    unsafe {
        gl::GenBuffers(1, &mut id);
    };

    assert_ne!(id, 0);

    return id;
}

/// Sets the vertex data in a buffer
/// # Arguments
/// * `id` - Buffer ID
/// * `data` - Data to upload
pub fn set_buffer_data<T>(id: u32, data: &Vec<T>) {
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (data.len() * std::mem::size_of::<T>()) as gl::types::GLsizeiptr,
            data.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW,
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    };
}

/// Bind a buffer
/// # Arguments
/// * `id` - Buffer ID
pub fn bind_buffer(id: u32) {
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, id);
    }
}

/// Generates a vertex attribute array on the GPU
pub fn generate_vertex_array() -> u32 {
    let mut id = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut id);
    };

    assert_ne!(id, 0);

    return id;
}

/// Bind an attribute array
/// # Arguments
/// * `id` - Vertex Array ID
pub fn bind_array(id: u32) {
    unsafe {
        gl::BindVertexArray(id);
    }
}

/// Set vertex attribute array
/// # Arguments
/// * `buffer` - Buffer vertex data is stored in
/// * `id` - Vertex Array ID
/// * `index` - Vertex Array Index to modify
/// * `size` - The number of components per vertex
pub fn set_vertex_array_pointer(buffer: u32, id: u32, index: u32, size: i32) {
    if size > 4 {
        panic!("Size must be 1, 2, 3, or 4");
    }

    unsafe {
        gl::BindVertexArray(id);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        gl::EnableVertexAttribArray(index);
        gl::VertexAttribPointer(index, size, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

        gl::BindVertexArray(0);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }
}

/// Generate a texture buffer
pub fn generate_texture() -> u32 {
    let mut id = 0;

    unsafe {
        gl::GenTextures(1, &mut id);
    };

    assert_ne!(id, 0);

    return id;
}

/// Bind a texture
/// # Arguments
/// * `id` - Texture ID
pub fn bind_texture(id: u32) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, id);
    }
}

/// Set texture data
/// # Arguments
/// * `id` - Texture ID
/// * `texture` - Texture Data
pub fn set_texture(id: u32, texture: image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as i32,
            texture.width() as i32,
            texture.height() as i32,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            texture.as_ptr() as *const gl::types::GLvoid,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    }
}

/// Set texture data directly from a file
/// # Arguments
/// * `file` - File path
pub fn create_texture_from_file(file: &str) -> u32 {
    // Try to load the texture
    let texture = match image::open(file) {
        Ok(image) => image.flipv().into_rgba8(),
        Err(message) => panic!("Image could not be loaded: {}", message),
    };

    // Create a texture and set the image data
    let id = generate_texture();
    set_texture(id, texture);

    return id;
}

/// Draw Triangles
/// # Arguments
/// * `vertex_count` - Number of vertices to draw
pub fn draw_triangles(vertex_count: u32) {
    unsafe { gl::DrawArrays(gl::TRIANGLES, 0, vertex_count as i32) }
}

/// Set the value of a vec3 uniform
/// # Arguments
/// * `uniform` - The name of the uniform to copy data to
/// * `program` - The shader program in use
/// * `data` - Data to copy to the uniform
pub fn set_uniform_float_vec3(uniform: &str, program: u32, data: &Vec<f32>) {
    unsafe {
        let location = gl::GetUniformLocation(program, CString::new(uniform).unwrap().as_ptr());
        gl::Uniform3fv(location, 1, data.as_ptr());
    }
}

/// Set the value of a vec2 uniform
/// # Arguments
/// * `uniform` - The name of the uniform to copy data to
/// * `program` - The shader program in use
/// * `data` - Data to copy to the uniform
pub fn set_uniform_float_vec2(uniform: &str, program: u32, data: &Vec<f32>) {
    unsafe {
        let location = gl::GetUniformLocation(program, CString::new(uniform).unwrap().as_ptr());
        gl::Uniform2fv(location, 1, data.as_ptr());
    }
}
