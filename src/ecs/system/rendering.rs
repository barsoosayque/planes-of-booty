use super::{
    super::{component::*, resource::*},
    render_sprite,
};
use crate::assets::AssetManager;
use crate::entity;
use crate::math::*;
use crate::ui::ImGuiSystem;
use ggez::{graphics, Context};
use specs::{Entities, Join, Read, ReadStorage, System, Write};

pub struct UiRenderSystem<'a>(pub &'a mut ggez::Context, pub &'a mut ImGuiSystem);
impl<'a> System<'a> for UiRenderSystem<'_> {
    type SystemData = (Read<'a, UiHub>, Read<'a, Inputs>, Write<'a, AssetManager>);

    fn run(&mut self, (ui_hub, inputs, mut assets): Self::SystemData) {
        // spawn selected debug item under cursor
        if let Some((sprite, width, height)) = ui_hub
            .debug_window
            .selected_entity
            .and_then(|id| entity::view(id, self.0, &mut assets))
        {
            render_sprite(
                self.0,
                &sprite.0,
                &inputs.mouse_pos.to_vector(),
                &Size2f::new(width, height),
            );
        }

        self.1.render(self.0);
    }
}

pub struct SpriteRenderSystem<'a>(pub &'a mut Context);
impl<'a> System<'a> for SpriteRenderSystem<'_> {
    type SystemData = (
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Movement>,
        ReadStorage<'a, DirectionalSprite>,
        ReadStorage<'a, Sprite>,
    );

    fn run(&mut self, (transforms, movements, dir_sprites, sprites): Self::SystemData) {
        for (transform, movement, sprite) in (&transforms, &movements, &dir_sprites).join() {
            let img = match Direction::from_vec2f(&movement.velocity) {
                Direction::North => &sprite.north.0,
                Direction::East => &sprite.east.0,
                Direction::South => &sprite.south.0,
                Direction::West => &sprite.west.0,
            };

            render_sprite(
                self.0,
                &img,
                &transform.pos,
                &Size2f::new(sprite.width, sprite.height),
            );
        }

        for (transform, sprite) in (&transforms, &sprites).join() {
            render_sprite(
                self.0,
                &sprite.asset.0,
                &transform.pos,
                &Size2f::new(sprite.width, sprite.height),
            );
        }
    }
}

pub struct DebugRenderSystem<'a>(pub &'a mut Context);
impl<'a> System<'a> for DebugRenderSystem<'_> {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Target>,
        ReadStorage<'a, SearchForTarget>,
        ReadStorage<'a, FollowTarget>,
    );

    fn run(&mut self, (entities, transforms, targets, searches, follows): Self::SystemData) {
        for (e, transform, target_opt, search_opt, follow_opt) in (
            &entities,
            &transforms,
            (&targets).maybe(),
            (&searches).maybe(),
            (&follows).maybe(),
        )
            .join()
        {
            if let Some(search) = search_opt {
                let circle = graphics::Mesh::new_circle(
                    self.0,
                    graphics::DrawMode::fill(),
                    Point2f::zero(),
                    search.radius,
                    0.5,
                    graphics::Color::from_rgba_u32(0xCC330044),
                )
                .unwrap();
                let param = graphics::DrawParam::default().dest(transform.pos.to_point());
                ggez::graphics::draw(self.0, &circle, param).unwrap();
            }
            if let Some(follow) = follow_opt {
                let circle = graphics::Mesh::new_circle(
                    self.0,
                    graphics::DrawMode::stroke(2.0),
                    Point2f::zero(),
                    follow.follow_distance,
                    0.5,
                    graphics::Color::from_rgba_u32(0xFFFFFFFF),
                )
                .unwrap();
                let param = graphics::DrawParam::default().dest(transform.pos.to_point());
                ggez::graphics::draw(self.0, &circle, param).unwrap();
            }

            let mut text = format!(
                "{:?}\nTransform({:.1}, {:.1})",
                e, transform.pos.x, transform.pos.y
            );
            if let Some(target) = target_opt {
                text.push_str(&format!("\n{:?}", target));
            }
            let text =
                graphics::TextFragment::from(text).color(graphics::Color::from_rgb_u32(0x00000000));
            let text = graphics::Text::new(text);

            let param = graphics::DrawParam::default()
                .dest((transform.pos + Vec2f::new(0.0, 30.0)).to_point());
            ggez::graphics::draw(self.0, &text, param).unwrap();
        }
    }
}
