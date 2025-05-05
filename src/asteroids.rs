use crate::bullet::Destructable;
use crate::health::{Dead, Health};
use crate::pickups::Material;
use crate::sampler::Sampler;
use crate::{GameState, Layer, assets};
use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind};
use bevy_tween::tween::IntoTarget;
use physics::linear_velocity;
use rand::Rng;
use std::time::Duration;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AsteroidSpawner(true))
            .add_systems(
                Update,
                (spawn_asteroids, handle_death, despawn_asteroids)
                    .run_if(in_state(GameState::Game)),
            )
            .add_systems(FixedUpdate, move_clusters.run_if(in_state(GameState::Game)));
    }
}

#[derive(Component)]
#[require(
    Destructable,
    CollisionLayers::new(Layer::Debris, [Layer::Bullet]),
)]
#[component(on_add = Self::add_hook)]
pub enum Asteroid {
    Big,
    Small,
}

impl Asteroid {
    pub fn add_hook(mut world: DeferredWorld, ctx: HookContext) {
        let asteroid = world.entity(ctx.entity).get::<Self>().unwrap();
        let server = world.get_resource::<AssetServer>().unwrap();

        let sprite = match asteroid {
            Self::Big => assets::sprite_rect16(server, assets::MISC_PATH, UVec2::new(1, 1)),
            Self::Small => assets::sprite_rect8(server, assets::MISC_PATH, UVec2::new(1, 3)),
        };
        let velocity = match asteroid {
            Self::Big => LinearVelocity(Vec2::NEG_Y * 18.),
            Self::Small => LinearVelocity(Vec2::NEG_Y * 22.),
        };
        let health = match asteroid {
            Self::Big => Health::full(15.0),
            Self::Small => Health::full(5.0),
        };

        world
            .commands()
            .entity(ctx.entity)
            .insert((sprite, velocity, health));
    }
}

#[derive(Resource)]
pub struct AsteroidSpawner(pub bool);

fn spawn_asteroids(
    mut commands: Commands,
    time: Res<Time<Physics>>,
    mut big_cooldown: Local<Option<Timer>>,
    mut small_cooldown: Local<Option<Timer>>,
    spawner: Res<AsteroidSpawner>,
) {
    if !spawner.0 {
        return;
    }

    const Y: f32 = crate::HEIGHT / 2. + 16.;
    const X: f32 = crate::WIDTH / 2. - 32.;

    let big_cooldown = big_cooldown.get_or_insert_with(|| Timer::from_seconds(5., TimerMode::Once));
    let small_cooldown =
        small_cooldown.get_or_insert_with(|| Timer::from_seconds(2., TimerMode::Once));

    big_cooldown.tick(time.delta());
    small_cooldown.tick(time.delta());

    if big_cooldown.finished() && rand::rng().random_bool(0.005) {
        let x = rand::rng().random_range(-X..X);
        commands.spawn((Asteroid::Big, Transform::from_xyz(x, Y, -10.)));
        big_cooldown.reset();
    }

    if small_cooldown.finished() && rand::rng().random_bool(0.01) {
        let x = rand::rng().random_range(-X..X);
        commands.spawn((Asteroid::Small, Transform::from_xyz(x, Y, -10.)));
        small_cooldown.reset();
    }
}

#[derive(Component)]
#[require(Visibility)]
pub struct MaterialCluster;

const MATERIAL_SPEED: f32 = 20.; // 20 meters per second

fn handle_death(
    asteroids: Query<(Entity, &GlobalTransform, &Asteroid), With<Dead>>,
    mut commands: Commands,
) {
    for (entity, transform, asteroid) in asteroids.iter() {
        commands.entity(entity).despawn();

        let children = match asteroid {
            Asteroid::Big => 10,
            Asteroid::Small => 5,
        };
        let transform = transform.compute_transform();

        let material_speed = 50.;
        let dur_variation = 0.75;

        let mut rng = rand::rng();
        let speeds = (0..material_speed as usize)
            .map(|speed| speed as f32)
            .collect::<Vec<_>>();
        let sampler = Sampler::linear(&speeds, 0.0, 1.0);

        let cluster = commands.spawn((MaterialCluster, transform)).id();
        for angle in 0..children {
            let angle = (angle as f32 / children as f32) * 2. * std::f32::consts::PI;
            let start = Vec2::from_angle(angle) * sampler.sample(&mut rng);

            let material = commands.spawn((Material, LinearVelocity::ZERO)).id();
            commands.entity(material).animation().insert_tween_here(
                Duration::from_secs_f32(
                    (1.5 + rng.random_range(-dur_variation..dur_variation)) * 0.75,
                ),
                EaseKind::CubicOut,
                material
                    .into_target()
                    .with(linear_velocity(start, Vec2::ZERO)),
            );
            commands.entity(cluster).add_child(material);
        }
    }
}

fn move_clusters(
    mut clusters: Query<&mut Transform, With<MaterialCluster>>,
    time: Res<Time<Physics>>,
) {
    for mut transform in clusters.iter_mut() {
        transform.translation.y -= MATERIAL_SPEED * time.delta_secs();
    }
}

fn despawn_asteroids(
    mut commands: Commands,
    asteroids: Query<(Entity, &Transform), With<Asteroid>>,
    clusters: Query<Entity, (With<MaterialCluster>, Without<Children>)>,
) {
    for (entity, transform) in asteroids.iter() {
        if transform.translation.y <= -crate::HEIGHT / 2. - 8. {
            commands.entity(entity).despawn();
        }
    }

    for entity in clusters.iter() {
        commands.entity(entity).despawn();
    }
}
