#version 330 core

precision mediump float;

uniform sampler2D texture_sampler;

in vec2 texture_coordinate;

out vec4 Color;

void main() {
    Color = vec4(1.0, 0.0, 1.0, 1.0);
}
