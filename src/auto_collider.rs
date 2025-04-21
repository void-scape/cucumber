use bevy::prelude::*;
use physics::prelude::*;

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

        let offset = Vec2::new(-size.x / 2.0, size.y / 2.0);
        commands
            .entity(entity)
            .insert(CollisionTrigger(Collider::from_rect(offset, size)));
    }
}

#[derive(Component, Default)]
pub struct ImageCollider;

fn insert_image_collider(
    q: Query<(Entity, &Sprite), With<ImageCollider>>,
    mut commands: Commands,
    images: Res<Assets<Image>>,
) {
    for (entity, sprite) in q.iter() {
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };

        let search_bounds = match sprite.rect {
            Some(rect) => rect,
            None => {
                let size = image.size();
                Rect::from_corners(Vec2::default(), Vec2::new(size.x as f32, size.y as f32))
            }
        };

        // let mut bounds = Rect::default();
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

        let bounds = Rect::from_corners(min, max);
        let bounds_size = bounds.size();
        let collider = Collider::from_rect(
            Vec2::new(
                bounds.min.x - bounds_size.x / 2.0,
                bounds.max.y + bounds_size.y / 2.0,
            ),
            bounds_size + 1.0,
        );
        commands
            .entity(entity)
            .insert((CollisionTrigger(collider), collider))
            .remove::<ImageCollider>();
    }
}
