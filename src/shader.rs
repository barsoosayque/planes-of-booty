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
