extern crate gl;
use gl::types::{GLenum, GLuint};

use std::fs;

use super::gl_util;

fn new_from_file(path: &str, kind: GLenum) -> GLuint {
    // Read the sourec file in as a string
    let source = match fs::read_to_string(path) {
        Ok(string) => string,
        Err(message) => panic!("Shader create failed: {}", message),
    };
}
