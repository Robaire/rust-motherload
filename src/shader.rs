extern crate gl;
use gl::types::{GLenum, GLuint};

use super::gl_util;

/// Create a shader program using a string as the source code
/// # Arguments
/// `source` - The shader source code string
/// `kind` - The kind of shader to create
pub fn new_from_string(source: String, kind: GLenum) -> Result<u32, String> {

    // Create a shader
    let shader_id = gl_util::create_shader(kind)?;

    // Compile the shader
    gl_util::compile_shader(shader_id, source);

    // Check if the shader compiled
    let status = gl_util::get_shader_parameter(shader_id, gl::COMPILE_STATUS);

    if status == 1 {
        return Ok(shader_id);
    } else {
        return Err(gl_util::get_shader_info_log(shader_id));
    }
}

/// Create a shader program using a file as the source code
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
