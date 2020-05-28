#version 150 core

uniform sampler2D t_Texture;
in vec2 v_Uv;
in vec4 v_Color;
out vec4 Target0;

layout (std140) uniform Globals {
    mat4 u_MVP;
};

layout (std140) uniform SilhouetteData {
    vec4 u_silhouette_color;
};

void main() {
    vec4 color = texture(t_Texture, v_Uv); 
    Target0 = vec4(u_silhouette_color.rgb * v_Color.rgb, color.a);
}
