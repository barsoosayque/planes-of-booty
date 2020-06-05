use crate::{
    assets::{Asset, ImageAsset},
    math::Point2f,
};
use gfx::{
    format::Rgba8,
    handle::{RenderTargetView, ShaderResourceView},
    memory::Typed,
    Factory,
};
use ggez::graphics::{self, BackendSpec, GlBackendSpec};
use imgui;
use imgui_gfx_renderer::*;
use std::cell::RefCell;

pub trait UiBuilder<D> {
    fn build<'ctx>(&mut self, ui: &mut imgui::Ui, ctx: &mut UiContext<'ctx>, data: D);
}

pub struct UiContext<'ctx>(
    &'ctx mut Renderer<Rgba8, <GlBackendSpec as BackendSpec>::Resources>,
    &'ctx mut ggez::Context,
);
impl UiContext<'_> {
    pub fn get_texture_id_for(&mut self, asset: &ImageAsset) -> imgui::TextureId {
        let id = imgui::TextureId::from(asset.id() as usize);
        if self.0.textures().get(id).is_some() {
            return id;
        }

        log::debug!("Associating a new ImageAsset with Imgui: {:?}", id);
        let resource_view = ShaderResourceView::new(asset.as_ref().raw_shader_resource_view().to_owned());
        let (factory, _, _, _, _) = graphics::gfx_objects(self.1);
        let sampler = factory.create_sampler(*asset.as_ref().sampler_info());
        self.0.textures().replace(id, (resource_view, sampler));
        id
    }
}
impl AsMut<ggez::Context> for UiContext<'_> {
    fn as_mut(&mut self) -> &mut ggez::Context { self.1 }
}

pub struct ImGuiSystem {
    imgui: imgui::Context,
    renderer: RefCell<Renderer<Rgba8, <GlBackendSpec as BackendSpec>::Resources>>,
    next_frame: RefCell<Option<imgui::Ui<'static>>>,
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
        Self { imgui: imgui, renderer: RefCell::new(renderer), next_frame: RefCell::new(None) }
    }

    pub fn update<D, B>(
        &mut self,
        ctx: &mut ggez::Context,
        delta: std::time::Duration,
        builder: &mut B,
        data: D,
    ) -> bool
    where
        B: UiBuilder<D>,
    {
        use ggez::input::mouse::{self, MouseButton};

        let mut io = self.imgui.io_mut();
        let window = graphics::window(ctx);

        // it's very important to round so we don't get blurry image
        let hidpi_factor = window.get_hidpi_factor().round();
        io.display_framebuffer_scale = [hidpi_factor as f32, hidpi_factor as f32];
        if let Some(logical_size) = window.get_inner_size() {
            // convert size using our rounded hidpi factor
            let rounded_size = logical_size.to_physical(window.get_hidpi_factor()).to_logical(hidpi_factor);
            io.display_size = [rounded_size.width as f32, rounded_size.height as f32];
        }

        let rounded_position =
            Point2f::from(ggez::input::mouse::position(ctx)) * window.get_hidpi_factor() as f32 / hidpi_factor as f32;
        io.mouse_pos = [rounded_position.x as f32, rounded_position.y as f32];

        io.mouse_down = [
            mouse::button_pressed(ctx, MouseButton::Left),
            mouse::button_pressed(ctx, MouseButton::Right),
            mouse::button_pressed(ctx, MouseButton::Middle),
            false,
            false,
        ];

        io.delta_time = delta.as_secs_f32();

        let mut ui = self.imgui.frame();
        let mut ctx = UiContext(self.renderer.get_mut(), ctx);
        builder.build(&mut ui, &mut ctx, data);
        let hovered = ui.is_window_hovered_with_flags(imgui::WindowHoveredFlags::all());

        unsafe {
            // bypass lifetime since it's not public and
            // limited to imgui context lifetime anyways
            self.next_frame.replace(Some(std::mem::transmute(ui)));
        }

        hovered
    }

    pub fn render(&self, ctx: &mut ggez::Context) {
        // consume next_frame and render it
        if let Some(ui) = self.next_frame.borrow_mut().take() {
            let (factory, _, encoder, _, render_target) = graphics::gfx_objects(ctx);
            let draw_data = ui.render();
            self.renderer
                .borrow_mut()
                .render(factory, encoder, &mut RenderTargetView::new(render_target), &draw_data)
                .unwrap();
        }
    }
}
