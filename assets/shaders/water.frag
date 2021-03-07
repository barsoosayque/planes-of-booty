#version 450

layout(location = 0) in vec2 v_Uv;
layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform WaterMaterial_color { vec4 color; };
layout(set = 2, binding = 1) uniform WaterMaterial_time { double time; };
layout(set = 2, binding = 2) uniform texture2D WaterMaterial_water_texture;
layout(set = 2, binding = 3) uniform sampler WaterMaterial_water_texture_sampler;

void main() {
    double animate = mod(time * 0.1, 1.0);

    vec4 tex = texture(sampler2D(WaterMaterial_water_texture, WaterMaterial_water_texture_sampler), 
                       v_Uv + vec2(animate, -animate));
    vec3 c = color.rgb + tex.rgb;
    
    o_Target = vec4(c.rgb, 0.25);
}