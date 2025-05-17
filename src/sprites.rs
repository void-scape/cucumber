use crate::bullet::emitter::{CrisscrossState, PulseTime, PulseTimer};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_tween::interpolate::rotation;
use bevy_tween::prelude::*;
use std::f32::EPSILON;
use std::f32::consts::PI;

pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_tile_sprites,
                (
                    spawn_sprite_bundles,
                    spawn_cell_sprites,
                    update_sprite_behavior,
                )
                    .chain(),
            ),
        );
    }
}

#[derive(Component)]
pub struct TiltSprite {
    pub path: &'static str,
    pub size: CellSize,
    //
    pub left: UVec2,
    pub center: UVec2,
    pub right: UVec2,
}

fn update_tile_sprites(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut new_sprites: Query<(Entity, &LinearVelocity, &TiltSprite), Without<Sprite>>,
    mut sprites: Query<(&LinearVelocity, &mut Sprite, &TiltSprite), Changed<LinearVelocity>>,
) {
    for (entity, velocity, tilt) in new_sprites.iter_mut() {
        let mut sprite = Sprite::from_image(server.load(tilt.path));
        assign_rect(&mut sprite, velocity, tilt);
        commands.entity(entity).insert(sprite);
    }

    for (velocity, mut sprite, tilt) in sprites.iter_mut() {
        assign_rect(&mut sprite, velocity, tilt);
    }
}

fn assign_rect(sprite: &mut Sprite, velocity: &LinearVelocity, tilt: &TiltSprite) {
    let cell = if velocity.0.x < -EPSILON {
        tilt.left
    } else if velocity.0.x > EPSILON {
        tilt.right
    } else {
        tilt.center
    };
    sprite.rect = Some(rect(tilt.size, cell));
}

#[derive(Clone, Copy, Component)]
#[require(Visibility)]
pub struct CellSprite {
    pub path: &'static str,
    pub size: CellSize,
    pub cell: UVec2,
    pub z: f32,
}

impl CellSprite {
    pub fn new8(path: &'static str, cell: UVec2) -> Self {
        Self {
            path,
            size: CellSize::Eight,
            cell,
            z: 0.,
        }
    }

    pub fn new16(path: &'static str, cell: UVec2) -> Self {
        Self {
            path,
            size: CellSize::Sixteen,
            cell,
            z: 0.,
        }
    }

    pub fn new24(path: &'static str, cell: UVec2) -> Self {
        Self {
            path,
            size: CellSize::TwentyFour,
            cell,
            z: 0.,
        }
    }
}

#[derive(Clone, Copy)]
pub enum CellSize {
    Eight,
    Sixteen,
    TwentyFour,
}

pub enum MultiSprite {
    Static(CellSprite),
    Dynamic {
        sprite: CellSprite,
        behavior: SpriteBehavior,
        position: Vec2,
    },
}

#[derive(Clone, Copy, Component)]
#[require(LinearVelocity, RigidBody::Kinematic)]
pub enum SpriteBehavior {
    Follow { lag: f32, speed: f32 },
    Crisscross,
}

#[derive(Component)]
#[relationship(relationship_target = BehaviorNodes)]
pub struct BehaviorRoot(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = BehaviorRoot, linked_spawn)]
pub struct BehaviorNodes(Vec<Entity>);

fn update_sprite_behavior(
    mut commands: Commands,
    time: Res<Time>,
    roots: Query<(Entity, &BehaviorNodes, &GlobalTransform)>,
    mut nodes: Query<
        (Entity, &mut Transform, &mut LinearVelocity, &SpriteBehavior),
        With<BehaviorRoot>,
    >,
    crisscross: Query<(&CrisscrossState, &PulseTimer), Changed<CrisscrossState>>,
) {
    for (root, child_nodes, transform) in roots.iter() {
        let mut iter = nodes.iter_many_mut(child_nodes.0.iter());
        let rp = transform.translation().xy();
        let rr = transform.rotation();

        while let Some((entity, mut transform, mut velocity, behavior)) = iter.fetch_next() {
            match behavior {
                SpriteBehavior::Follow { lag, speed } => {
                    let p = transform.translation.xy();
                    let to_root = (rp - p).clamp_length(0., *lag) / *lag;
                    velocity.0 = to_root * *speed;

                    transform.rotation = transform.rotation.rotate_towards(
                        rr,
                        transform.rotation.angle_between(rr) * time.delta_secs(),
                    );
                }
                SpriteBehavior::Crisscross => {
                    transform.translation.x = rp.x;
                    transform.translation.y = rp.y;
                    if let Ok((state, timer)) = crisscross.get(root) {
                        match state {
                            CrisscrossState::Plus => {
                                commands.entity(entity).animation().insert_tween_here(
                                    Duration::from_secs_f32(timer.wait_time() / 1.2),
                                    EaseKind::Linear,
                                    entity.into_target().with(rotation(
                                        Quat::from_rotation_z(PI / 4.),
                                        Quat::default(),
                                    )),
                                );
                            }
                            CrisscrossState::Cross => {
                                commands.entity(entity).animation().insert_tween_here(
                                    Duration::from_secs_f32(timer.wait_time() / 1.2),
                                    EaseKind::Linear,
                                    entity.into_target().with(rotation(
                                        Quat::default(),
                                        Quat::from_rotation_z(PI / 4.),
                                    )),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct SpriteBundle(Vec<MultiSprite>);

impl SpriteBundle {
    pub fn new(sprites: impl IntoIterator<Item = MultiSprite>) -> Self {
        Self(sprites.into_iter().collect())
    }
}

#[derive(Component)]
struct Populated;

fn spawn_sprite_bundles(
    mut commands: Commands,
    bundles: Query<(Entity, &SpriteBundle, &GlobalTransform), Without<Populated>>,
) {
    for (entity, bundle, gt) in bundles.iter() {
        commands.entity(entity).insert(Populated);
        for sprite in bundle.0.iter() {
            match sprite {
                MultiSprite::Static(sprite) => {
                    commands.entity(entity).with_child(*sprite);
                }
                MultiSprite::Dynamic {
                    sprite,
                    behavior,
                    position,
                } => {
                    let mut new_transform = gt.compute_transform();
                    new_transform.translation += position.extend(sprite.z);
                    commands.spawn((*sprite, *behavior, BehaviorRoot(entity), new_transform));
                }
            }
        }
    }
}

fn spawn_cell_sprites(
    mut commands: Commands,
    server: Res<AssetServer>,
    sprites: Query<(Entity, &CellSprite, Option<&Transform>)>,
) {
    for (entity, sprite, transform) in sprites.iter() {
        commands
            .entity(entity)
            .insert(sprite_rect(&server, sprite.path, sprite.size, sprite.cell))
            .remove::<CellSprite>();
        if transform.is_none() {
            commands
                .entity(entity)
                .insert(Transform::from_xyz(0., 0., sprite.z));
        }
    }
}

pub fn sprite_rect(
    server: &AssetServer,
    path: &'static str,
    size: CellSize,
    cell: UVec2,
) -> Sprite {
    Sprite {
        image: server.load(path),
        rect: Some(rect(size, cell)),
        ..Default::default()
    }
}

fn rect(size: CellSize, cell: UVec2) -> Rect {
    let size = match size {
        CellSize::Eight => 8.,
        CellSize::Sixteen => 16.,
        CellSize::TwentyFour => 24.,
    };
    Rect::from_corners(cell.as_vec2() * size, (cell.as_vec2() + 1.) * size)
}
