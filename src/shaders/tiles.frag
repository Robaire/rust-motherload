#version 330 core

precision mediump float;

in vec3 texture_vertex;

uniform sampler2DArray texture_sampler;

out vec4 Color;

void main() {
    Color = texture(texture_sampler, texture_vertex);
}
