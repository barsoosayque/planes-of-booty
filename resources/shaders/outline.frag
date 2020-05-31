#version 150 core

uniform sampler2D t_Texture;
in vec2 v_Uv;
in vec4 v_Color;
out vec4 Target0;

layout (std140) uniform OutlineData {
    vec4 u_outline_color;
    vec2 u_step;
};

void main() {
    vec4 color = texture(t_Texture, v_Uv); 
    if (color.a > 0.8) {
        Target0 = color * v_Color;
    } else {
        float test = texture(t_Texture, vec2(v_Uv.x, v_Uv.y + u_step.y)).a
            + texture(t_Texture, vec2(v_Uv.x, v_Uv.y - u_step.y)).a
            + texture(t_Texture, vec2(v_Uv.x + u_step.x, v_Uv.y)).a
            + texture(t_Texture, vec2(v_Uv.x - u_step.x, v_Uv.y)).a;
        Target0 = mix(vec4(0.0), u_outline_color, float(test > 0.0));
    }
}
