extern crate gl;
use gl::types::{GLenum, GLuint};

use super::gl_util;

pub fn new_from_file(path: &str, kind: GLenum) -> Result<u32, String> {
    // Read the sourec file in as a string
    let source = match std::fs::read_to_string(path) {
        Ok(string) => string,
        Err(message) => panic!("Shader create failed: {}", message),
    };

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
