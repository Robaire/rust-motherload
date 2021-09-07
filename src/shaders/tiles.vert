#version 330 core

layout(location = 0) in vec2 tile_vertex;
layout(location = 1) in vec3 texture_vertex;

uniform mat4 projection;

out vec3 texture_vertex_frag;

void main() {

    // Pass the texture coordinates along to the fragment shader
    texture_vertex_frag = texture_vertex;

    // Translate the tile vertex according to the camera view
    gl_Position = projection * vec4(tile_vertex, 0.0, 1.0);
}
