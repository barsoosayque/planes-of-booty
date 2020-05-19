use gfx::{format::Rgba8, handle::RenderTargetView, memory::Typed};
use crate::math::Point2f;
use ggez::graphics::{self, BackendSpec, GlBackendSpec};
use imgui;
use imgui_gfx_renderer::*;
use std::iter::IntoIterator;

pub trait UiBuilder {
    fn build(&self, ui: &mut imgui::Ui);
}

pub struct ImGuiSystem {
    imgui: imgui::Context,
    renderer: Renderer<Rgba8, <GlBackendSpec as BackendSpec>::Resources>,
}

impl ImGuiSystem {
    pub fn new(ctx: &mut ggez::Context) -> ImGuiSystem {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        let (factory, device, _, _, _) = graphics::gfx_objects(ctx);
        let shaders = {
            let version = device.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                Shaders::GlSl130
            } else {
                Shaders::GlSl110
            }
        };
        let renderer = Renderer::init(&mut imgui, &mut *factory, shaders).unwrap();
        Self { imgui, renderer }
    }

    pub fn update(&mut self, ctx: &ggez::Context, delta: std::time::Duration) {
        use ggez::input::mouse::{self, MouseButton};

        let mut io = self.imgui.io_mut();
        let window = graphics::window(ctx);

        // it's very important to round so we don't get blurry image
        let hidpi_factor = window.get_hidpi_factor().round();
        io.display_framebuffer_scale = [hidpi_factor as f32, hidpi_factor as f32];
        if let Some(logical_size) = window.get_inner_size() {
            // convert size using our rounded hidpi factor
            let rounded_size = logical_size
                .to_physical(window.get_hidpi_factor())
                .to_logical(hidpi_factor);
            io.display_size = [rounded_size.width as f32, rounded_size.height as f32];
        }

        let rounded_position = Point2f::from(ggez::input::mouse::position(ctx))
            * window.get_hidpi_factor() as f32
            / hidpi_factor as f32;
        io.mouse_pos = [rounded_position.x as f32, rounded_position.y as f32];

        io.mouse_down = [
            mouse::button_pressed(ctx, MouseButton::Left),
            mouse::button_pressed(ctx, MouseButton::Right),
            mouse::button_pressed(ctx, MouseButton::Middle),
            false,
            false,
        ];

        io.delta_time = delta.as_secs_f32();
    }

    pub fn render<I, U>(&mut self, ctx: &mut ggez::Context, builders_iter: I)
    where
        U: UiBuilder,
        I: IntoIterator<Item = U>,
    {
        let mut ui = self.imgui.frame();
        for builder in builders_iter {
            builder.build(&mut ui);
        }

        let (factory, _, encoder, _, render_target) = graphics::gfx_objects(ctx);
        let draw_data = ui.render();
        self.renderer
            .render(
                factory,
                encoder,
                &mut RenderTargetView::new(render_target),
                &draw_data,
            )
            .unwrap();
    }
}
