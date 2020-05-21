use super::{
    super::{component::*, resource::*},
    render_fill_circle, render_line, render_sprite, render_stroke_circle, render_polygon
};
use crate::assets::AssetManager;
use crate::entity;
use crate::math::*;
use crate::ui::ImGuiSystem;
use ggez::{graphics, Context};
use specs::{Entities, Join, Read, ReadExpect, ReadStorage, System, Write};

pub struct UiRenderSystem<'a>(pub &'a mut ggez::Context, pub &'a mut ImGuiSystem);
impl<'a> System<'a> for UiRenderSystem<'_> {
    type SystemData = (Read<'a, UiHub>, Read<'a, Inputs>, Write<'a, AssetManager>);

    fn run(&mut self, (ui_hub, inputs, mut assets): Self::SystemData) {
        // spawn selected debug item under cursor
        if let Some((sprite, size)) = ui_hub
            .debug_window
            .selected_entity
            .and_then(|id| entity::view(id, self.0, &mut assets))
        {
            render_sprite(self.0, &sprite.0, &inputs.mouse_pos.to_vector(), &size);
        }

        self.1.render(self.0);
    }
}

pub struct SpriteRenderSystem<'a>(pub &'a mut Context);
impl<'a> System<'a> for SpriteRenderSystem<'_> {
    type SystemData = (
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Movement>,
        ReadStorage<'a, Sprite>,
    );

    fn run(&mut self, (transforms, movements, sprites): Self::SystemData) {
        for (transform, movement, sprite) in (&transforms, &movements, &sprites).join() {
            match &sprite.asset {
                SpriteAsset::Single { value } => {
                    render_sprite(self.0, &value.0, &transform.pos, &sprite.size);
                }
                SpriteAsset::Directional {
                    north,
                    east,
                    south,
                    west,
                } => {
                    let img = match Direction::from_vec2f(&movement.velocity) {
                        Direction::North => &north.0,
                        Direction::East => &east.0,
                        Direction::South => &south.0,
                        Direction::West => &west.0,
                    };

                    render_sprite(self.0, &img, &transform.pos, &sprite.size);
                }
            }
        }
    }
}

pub struct DebugTargetRenderSystem<'a>(pub &'a mut Context);
impl<'a> System<'a> for DebugTargetRenderSystem<'_> {
    type SystemData = (
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Target>,
        ReadStorage<'a, SearchForTarget>,
        ReadStorage<'a, FollowTarget>,
    );

    fn run(&mut self, (transforms, targets, searches, follows): Self::SystemData) {
        let join = (
            &transforms,
            &targets,
            (&searches).maybe(),
            (&follows).maybe(),
        )
            .join();

        for (transform, target, search, follow) in join {
            let pos = transform.pos.to_point();
            if let Some(target_e) = target.target {
                if let Some(follow) = follow {
                    let target_pos = transforms.get(target_e).unwrap().pos;

                    // if there is target and this entity is following it
                    render_stroke_circle(self.0, &pos, follow.follow_distance, 2.0, 0xFC2F2FCC);
                    render_stroke_circle(self.0, &pos, follow.keep_distance, 2.0, 0x9BD644CC);
                    render_line(
                        self.0,
                        &[transform.pos.to_point(), target_pos.to_point()],
                        2.0,
                        0xFC53A7CC,
                    );
                }
            } else if let Some(search) = search {
                // if no target and this entity is able to search for a target
                render_fill_circle(self.0, &pos, search.radius, 0xFC2F2F33);
            }
        }
    }
}

pub struct DebugInfoRenderSystem<'a>(pub &'a mut Context);
impl<'a> System<'a> for DebugInfoRenderSystem<'_> {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Target>,
    );

    fn run(&mut self, (entities, transforms, targets): Self::SystemData) {
        for (e, transform, target_opt) in (&entities, &transforms, (&targets).maybe()).join() {
            let mut text = format!(
                "{:?}\nTransform({:.1}, {:.1})",
                e, transform.pos.x, transform.pos.y
            );
            if let Some(target) = target_opt {
                text.push_str(&format!("\n{:?}", target));
            }
            let color = graphics::Color::from_rgb_u32(0x00000000);
            let text = graphics::TextFragment::from(text).color(color);
            let text = graphics::Text::new(text);

            let param = graphics::DrawParam::default()
                .dest((transform.pos + Vec2f::new(0.0, 30.0)).to_point());
            ggez::graphics::draw(self.0, &text, param).unwrap();
        }
    }
}

pub struct DebugPhysicRenderSystem<'a>(pub &'a mut Context);
impl<'a> System<'a> for DebugPhysicRenderSystem<'_> {
    type SystemData = ReadExpect<'a, PhysicWorld>;

    fn run(&mut self, world: Self::SystemData) {
        use nphysics2d::ncollide2d::shape::ConvexPolygon;
        for (_, collider) in world.colliders.iter() {
            if let Some(polygon) = collider.shape().as_shape::<ConvexPolygon<f32>>() {
                let points: Vec<Point2f> = polygon
                    .points()
                    .iter()
                    .map(|p| Point2f::new(p[0], p[1]))
                    .collect();
                render_polygon(self.0, &points, 0x05FC19AA);
            }
        }
    }
}
