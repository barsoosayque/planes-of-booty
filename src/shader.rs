use gfx::{self, *};

pub trait ShaderInName {
    fn name() -> &'static str;
}

// Data for silhouette shader
gfx_defines! {
    constant Silhouette {
        silhouette_color: [f32; 4] = "u_silhouette_color",
    }
}
impl Default for Silhouette {
    fn default() -> Self { Self { silhouette_color: [1.0, 1.0, 1.0, 1.0] } }
}
impl ShaderInName for Silhouette {
    fn name() -> &'static str { "SilhouetteData" }
}

// Data for outline shader
gfx_defines! {
    constant Outline {
        outline_color: [f32; 4] = "u_outline_color",
        step: [f32; 2] = "u_step",
    }
}
impl Default for Outline {
    fn default() -> Self { Self { outline_color: [1.0, 1.0, 1.0, 1.0], step: [0.0, 0.0] } }
}
impl ShaderInName for Outline {
    fn name() -> &'static str { "OutlineData" }
}
