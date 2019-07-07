#version 450

layout(location = 0) in vec3 a_Pos;
layout(location = 1) in vec3 a_Nor;
layout(location = 2) in vec2 a_UV;
layout(location = 0) flat out vec4 v_Color;

void main() {
    v_Color = vec4(0.3,0.3, 0.0, 1.0);

    gl_Position = vec4(a_Pos*0.1,1.0);
}
