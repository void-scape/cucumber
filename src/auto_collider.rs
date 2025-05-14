use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_optix::debug::{DebugCircle, DebugRect};

pub struct AutoColliderPlugin;

impl Plugin for AutoColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (insert_collider, insert_image_collider));
    }
}

#[derive(Component, Default)]
pub struct AutoCollider;

fn insert_collider(
    q: Query<(Entity, &Sprite), (Added<AutoCollider>, Without<Collider>)>,
    mut commands: Commands,
) {
    for (entity, sprite) in q.iter() {
        let Some(size) = sprite.rect.map(|r| r.size()) else {
            continue;
        };

        commands
            .entity(entity)
            .insert(Collider::rectangle(size.x, size.y));
    }
}

#[derive(Component, Default)]
pub struct ImageCollider;

fn insert_image_collider(
    sprites: Query<(Entity, &Sprite), (With<ImageCollider>, Without<DebugRect>)>,
    debug_rects: Query<(Entity, &DebugRect), With<ImageCollider>>,
    debug_circles: Query<(Entity, &DebugCircle), With<ImageCollider>>,
    mut commands: Commands,
    images: Res<Assets<Image>>,
) {
    for (entity, debug_circle) in debug_circles.iter() {
        commands
            .entity(entity)
            .insert(Collider::circle(debug_circle.radius))
            .remove::<ImageCollider>();
    }

    for (entity, debug_rect) in debug_rects.iter() {
        let size = debug_rect.rect.size();
        commands
            .entity(entity)
            .insert(Collider::rectangle(size.x, size.y))
            .remove::<ImageCollider>();
    }

    for (entity, sprite) in sprites.iter() {
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };

        let search_bounds = match sprite.rect {
            Some(rect) => rect,
            None => Rect::from_corners(Vec2::ZERO, image.size_f32()),
        };

        let mut min = Vec2::default();
        let mut max = Vec2::default();

        // leftmost
        'outer: for x in search_bounds.min.x as u32..search_bounds.max.x as u32 {
            for y in search_bounds.min.y as u32..search_bounds.max.y as u32 {
                if let Ok(color) = image.get_color_at(x, y) {
                    if !color.is_fully_transparent() {
                        min.x = x as f32 - search_bounds.min.x;
                        break 'outer;
                    }
                }
            }
        }

        // rightmost
        'outer: for x in (search_bounds.min.x as u32..search_bounds.max.x as u32).rev() {
            for y in search_bounds.min.y as u32..search_bounds.max.y as u32 {
                if let Ok(color) = image.get_color_at(x, y) {
                    if !color.is_fully_transparent() {
                        max.x = x as f32 - search_bounds.min.x;
                        break 'outer;
                    }
                }
            }
        }

        // topmost
        'outer: for y in search_bounds.min.y as u32..search_bounds.max.y as u32 {
            for x in search_bounds.min.x as u32..search_bounds.max.x as u32 {
                if let Ok(color) = image.get_color_at(x, y) {
                    if !color.is_fully_transparent() {
                        max.y = y as f32 - search_bounds.min.y;
                        break 'outer;
                    }
                }
            }
        }

        // bottommost
        'outer: for y in (search_bounds.min.y as u32..search_bounds.max.y as u32).rev() {
            for x in search_bounds.min.x as u32..search_bounds.max.x as u32 {
                if let Ok(color) = image.get_color_at(x, y) {
                    if !color.is_fully_transparent() {
                        min.y = -(y as f32 - search_bounds.min.y);
                        break 'outer;
                    }
                }
            }
        }

        if sprite.flip_y {
            std::mem::swap(&mut min.y, &mut max.y);
        }

        if sprite.flip_x {
            std::mem::swap(&mut min.x, &mut max.x);
        }

        let bounds = Rect::from_corners(min, max);
        let size = bounds.size();
        commands
            .entity(entity)
            .insert(Collider::rectangle(size.x, size.y))
            .remove::<ImageCollider>();
    }
}
