use bevy::prelude::*;
use physics::prelude::*;

pub struct AutoColliderPlugin;

impl Plugin for AutoColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, insert_collider);
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
