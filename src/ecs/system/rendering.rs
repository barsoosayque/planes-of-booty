use super::{
    super::{component::*, resource::*},
    render_fill_circle, render_fill_rect, render_fill_sprite, render_line, render_polygon, render_sprite,
    render_stroke_circle,
};
use crate::{assets::*, entity, math::*, shader, ui::ImGuiSystem};
use ggez::{graphics, Context};
use itertools::Itertools;
use specs::{Entities, Join, Read, ReadExpect, ReadStorage, System, Write};

pub struct ParticleRenderSystem<'a>(pub &'a mut Context);
impl<'a> System<'a> for ParticleRenderSystem<'a> {
    type SystemData =
        (ReadStorage<'a, Transform>, ReadStorage<'a, ParticleProperties>, ReadStorage<'a, SharedParticleDef>);

    fn run(&mut self, (transforms, props, defs): Self::SystemData) {
        let groups = (&transforms, &props, &defs)
            .join()
            .into_iter()
            .sorted_by_key(|(_, _, def)| def.spritesheet.id())
            .group_by(|(_, _, def)| &def.spritesheet);
        for (asset, group) in groups.into_iter() {
            let mut batch = graphics::spritebatch::SpriteBatch::new(asset.as_ref().as_ref().clone());
            for (transform, prop, def) in group {
                let scale = Vec2f::new(
                    (def.size.width * def.sheet_width as f32) / asset.width() as f32,
                    (def.size.height * def.sheet_height as f32) / asset.height() as f32,
                );

                let (frame_x, frame_y) = (prop.current_frame % def.sheet_width, prop.current_frame / def.sheet_width);
                let (frame_w, frame_h) = (1.0 / def.sheet_width as f32, 1.0 / def.sheet_height as f32);

                let param = graphics::DrawParam::default()
                    .scale(scale)
                    .offset(Point2f::new(0.5, 0.5))
                    .src([frame_x as f32 * frame_w, frame_y as f32 * frame_h, frame_w, frame_h].into())
                    .dest(transform.pos.to_point());
                batch.add(param);
            }
            graphics::draw(self.0, &batch, graphics::DrawParam::default()).unwrap();
        }
    }
}

pub struct UiRenderSystem<'a>(pub &'a mut ggez::Context, pub &'a mut ImGuiSystem);
impl<'a> System<'a> for UiRenderSystem<'_> {
    type SystemData = (ReadStorage<'a, Sprite>, Read<'a, UiHub>, Read<'a, Inputs>, Write<'a, AssetManager>);

    fn run(&mut self, (sprites, ui_hub, inputs, mut assets): Self::SystemData) {
        // spawn selected debug item under cursor
        if let Some((sprite, size)) =
            ui_hub.debug_window.selected_entity.and_then(|id| entity::view(id, self.0, &mut assets))
        {
            render_sprite(self.0, &sprite, &inputs.mouse_pos.to_vector(), &Angle2f::zero(), &size);
        }

        self.1.render(self.0);

        if let Some(Sprite { asset: SpriteAsset::Single { value }, .. }) =
            ui_hub.inventory_window.dragging_item().and_then(|item| sprites.get(item))
        {
            render_sprite(self.0, &value, &inputs.mouse_pos.to_vector(), &Angle2f::zero(), &Size2f::new(50.0, 50.0));
        }
    }
}

pub struct SpriteRenderSystem<'a>(pub &'a mut Context);
impl<'a> System<'a> for SpriteRenderSystem<'_> {
    type SystemData = (
        Entities<'a>,
        Write<'a, AssetManager>,
        Read<'a, InteractionCache>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Sprite>,
        ReadStorage<'a, SpriteBlink>,
        ReadStorage<'a, Directional>,
    );

    fn run(
        &mut self,
        (entities, mut assets, interaction, transforms, sprites, blinks, directionals): Self::SystemData,
    ) {
        let mut stream =
            (&entities, &transforms, &sprites, (&directionals).maybe(), (&blinks).maybe()).join().collect::<Vec<_>>();
        stream.sort_unstable_by(|t1, t2| {
            (t1.1.pos.y + t1.2.size.height * 0.5)
                .partial_cmp(&(t2.1.pos.y + t2.2.size.height * 0.5))
                .unwrap_or(std::cmp::Ordering::Less)
        });
        for (e, transform, sprite, directional_opt, blink_opt) in stream.into_iter() {
            let img = match &sprite.asset {
                SpriteAsset::Single { value } => Some(value),
                SpriteAsset::Directional { north, east, south, west } => {
                    if let Some(Directional { direction }) = directional_opt {
                        Some(directional!(direction => north, east, south, west))
                    } else {
                        None
                    }
                },
            };
            if let Some(img) = img {
                let _lock = if blink_opt.is_some() {
                    let s = assets.get::<ShaderAsset<shader::Silhouette>>("/shaders/silhouette.frag", self.0).unwrap();
                    Some(graphics::use_shader(self.0, &s))
                } else if interaction.near_inventory == Some(e) || interaction.near_level_changer == Some(e) {
                    let s = assets.get::<ShaderAsset<shader::Outline>>("/shaders/outline.frag", self.0).unwrap();
                    s.send(self.0, shader::Outline {
                        step: [4.0 / img.width() as f32, 4.0 / img.height() as f32],
                        ..shader::Outline::default()
                    })
                    .unwrap();
                    Some(graphics::use_shader(self.0, &s))
                } else {
                    None
                };

                render_sprite(self.0, &img, &transform.pos, &transform.rotation, &sprite.size);
            }
        }
    }
}

pub struct MapRenderingSystem<'a>(pub &'a mut Context);
impl MapRenderingSystem<'_> {
    const TILE: f32 = 100.0;
    const TILE2: f32 = Self::TILE * 2.0;
    const WATER_TILE: f32 = 50.0;
}
impl<'a> System<'a> for MapRenderingSystem<'_> {
    type SystemData = (Read<'a, Camera>, Write<'a, AssetManager>, Read<'a, Arena>);

    fn run(&mut self, (camera, mut assets, arena): Self::SystemData) {
        let size = graphics::window(self.0).get_inner_size().unwrap();
        let space = assets.get::<ImageAsset>("/sprites/map/space.png", self.0).unwrap();
        let water = assets.get::<ImageAsset>("/sprites/map/water.png", self.0).unwrap();

        let parallax_offset = Vec2f::new((camera.pos.x * -0.1) % Self::TILE, (camera.pos.y * -0.1) % Self::TILE);
        render_fill_sprite(
            self.0,
            &space,
            &(camera.pos + parallax_offset),
            &Angle2f::zero(),
            &Size2f::new(Self::TILE, Self::TILE),
            &Size2f::new(size.width as f32 + Self::TILE2, size.height as f32 + Self::TILE2),
        );

        render_fill_sprite(
            self.0,
            &water,
            &Vec2f::zero(),
            &Angle2f::zero(),
            &Size2f::new(Self::WATER_TILE, Self::WATER_TILE),
            &arena.size,
        );
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
        let join = (&transforms, &targets, (&searches).maybe(), (&follows).maybe()).join();

        for (transform, target, search, follow) in join {
            let pos = transform.pos.to_point();
            if let Some(target_e) = target.target {
                if let Some(follow) = follow {
                    let target_pos = transforms.get(target_e).unwrap().pos;

                    // if there is target and this entity is following it
                    render_stroke_circle(self.0, &pos, follow.follow_distance, 2.0, 0xFC2F2FCC);
                    render_stroke_circle(self.0, &pos, follow.keep_distance, 2.0, 0x9BD644CC);
                    render_line(self.0, &[transform.pos.to_point(), target_pos.to_point()], 2.0, 0xFC53A7CC);
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
    type SystemData = (Entities<'a>, ReadStorage<'a, Transform>, ReadStorage<'a, Target>, ReadStorage<'a, HealthPool>);

    fn run(&mut self, (entities, transforms, targets, hpools): Self::SystemData) {
        for (e, transform, target_opt, hpool_opt) in
            (&entities, &transforms, (&targets).maybe(), (&hpools).maybe()).join()
        {
            let mut text = format!("{:?}\nTransform({:.1}, {:.1})", e, transform.pos.x, transform.pos.y);
            if let Some(target) = target_opt {
                text.push_str(&format!("\n{:?}", target));
            }
            if let Some(hpool) = hpool_opt {
                text.push_str(&format!("\n{:?}", hpool));
            }
            let color = graphics::Color::from_rgb_u32(0x00000000);
            let text = graphics::TextFragment::from(text).color(color);
            let text = graphics::Text::new(text);

            let param = graphics::DrawParam::default().dest((transform.pos + Vec2f::new(0.0, 30.0)).to_point());
            ggez::graphics::draw(self.0, &text, param).unwrap();
        }
    }
}

pub struct DebugPhysicRenderSystem<'a>(pub &'a mut Context);
impl<'a> System<'a> for DebugPhysicRenderSystem<'_> {
    type SystemData = ReadExpect<'a, PhysicWorld>;

    fn run(&mut self, world: Self::SystemData) {
        use nphysics2d::ncollide2d::shape::{ConvexPolygon, Cuboid};
        for (_, collider) in world.colliders.iter() {
            let body_pos = collider.position().translation.vector;
            if let Some(polygon) = collider.shape().as_shape::<ConvexPolygon<f32>>() {
                let points: Vec<Point2f> =
                    polygon.points().iter().map(|p| Point2f::new(body_pos[0] + p[0], body_pos[1] + p[1])).collect();
                render_polygon(self.0, &points, 0x05FC19AA);
            }
            if let Some(cuboid) = collider.shape().as_shape::<Cuboid<f32>>() {
                let half_extents = cuboid.half_extents();
                render_fill_rect(
                    self.0,
                    // why div 2 ??????????????????????
                    &Point2f::new(body_pos[0] * 0.5, body_pos[1] * 0.5),
                    &Size2f::new(half_extents.x * 2.0, half_extents.y * 2.0),
                    0x05FC19AA,
                );
            }
        }
    }
}
