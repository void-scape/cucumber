use crate::bullet::Destructable;
use crate::health::{Dead, Health};
use crate::sprites::{self, CellSize};
use crate::{GameState, Layer, assets};
use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use rand::Rng;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnCluster>()
            .insert_resource(AsteroidSpawner(true))
            .add_systems(OnEnter(GameState::Restart), restart)
            .add_systems(
                Update,
                (handle_death, despawn_asteroids, spawn_clusters).run_if(in_state(GameState::Game)),
            )
            .add_systems(
                FixedUpdate,
                (spawn_asteroids, move_clusters).run_if(in_state(GameState::Game)),
            );
    }
}

fn restart(
    mut commands: Commands,
    asteroids: Query<Entity, With<Asteroid>>,
    clusters: Query<Entity, With<MaterialCluster>>,
) {
    for entity in asteroids.iter().chain(clusters.iter()) {
        commands.entity(entity).despawn();
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
            Self::Big => sprites::sprite_rect(
                server,
                assets::MISC_PATH,
                CellSize::Sixteen,
                UVec2::new(1, 1),
            ),
            Self::Small => {
                sprites::sprite_rect(server, assets::MISC_PATH, CellSize::Eight, UVec2::new(1, 3))
            }
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
    time: Res<Time>,
    mut big_cooldown: Local<Option<Timer>>,
    mut small_cooldown: Local<Option<Timer>>,
    spawner: Res<AsteroidSpawner>,
) {
    return;
    //if !spawner.0 {
    //    return;
    //}

    const Y: f32 = crate::HEIGHT / 2. + 16.;
    const X: f32 = crate::WIDTH / 2. - 32.;

    let big_cooldown = big_cooldown.get_or_insert_with(|| Timer::from_seconds(5., TimerMode::Once));
    let small_cooldown =
        small_cooldown.get_or_insert_with(|| Timer::from_seconds(2., TimerMode::Once));

    big_cooldown.tick(time.delta());
    small_cooldown.tick(time.delta());

    if big_cooldown.finished() && rand::rng().random_bool(0.001) {
        let x = rand::rng().random_range(-X..X);
        commands.spawn((Asteroid::Big, Transform::from_xyz(x, Y, -10.)));
        big_cooldown.reset();
    }

    if small_cooldown.finished() && rand::rng().random_bool(0.005) {
        let x = rand::rng().random_range(-X..X);
        commands.spawn((Asteroid::Small, Transform::from_xyz(x, Y, -10.)));
        small_cooldown.reset();
    }
}

#[derive(Event)]
pub struct SpawnCluster {
    pub parts: usize,
    pub shield: usize,
    pub position: Vec2,
}

#[derive(Component)]
#[require(Visibility)]
pub struct MaterialCluster;

pub const MATERIAL_SPEED: f32 = 20.;

fn handle_death(
    mut commands: Commands,
    mut writer: EventWriter<SpawnCluster>,
    asteroids: Query<(Entity, &GlobalTransform, &Asteroid), With<Dead>>,
) {
    for (entity, transform, asteroid) in asteroids.iter() {
        commands.entity(entity).despawn();
        writer.write(SpawnCluster {
            parts: match asteroid {
                Asteroid::Big => 10,
                Asteroid::Small => 5,
            },
            shield: 0,
            position: transform.compute_transform().translation.xy(),
        });
    }
}

fn spawn_clusters(mut commands: Commands, mut reader: EventReader<SpawnCluster>) {
    for event in reader.read() {
        //let material_speed = 50.;
        //let dur_variation = 0.75;
        //
        //let mut rng = rand::rng();
        //let speeds = (0..material_speed as usize)
        //    .map(|speed| speed as f32)
        //    .collect::<Vec<_>>();
        //let sampler = Sampler::linear(&speeds, 0.0, 1.0);
        //
        //let cluster = commands
        //    .spawn((
        //        MaterialCluster,
        //        Transform::from_translation(event.position.extend(0.)),
        //    ))
        //    .id();
        //
        //let materials = event.parts + event.shield;
        //for angle in 0..event.parts {
        //    let angle = (angle as f32 / materials as f32) * 2. * std::f32::consts::PI
        //        + rng.random_range(-0.5..0.5);
        //    let start = Vec2::from_angle(angle) * sampler.sample(&mut rng);
        //
        //    let material = commands.spawn((Material::Parts, LinearVelocity::ZERO)).id();
        //    commands.entity(material).animation().insert_tween_here(
        //        Duration::from_secs_f32(
        //            (1.5 + rng.random_range(-dur_variation..dur_variation)) * 0.75,
        //        ),
        //        EaseKind::CubicOut,
        //        material
        //            .into_target()
        //            .with(linear_velocity(start, Vec2::ZERO)),
        //    );
        //    commands.entity(cluster).add_child(material);
        //}
        //
        //for angle in 0..event.shield {
        //    let angle = (angle as f32 / materials as f32) * 2. * std::f32::consts::PI
        //        + rng.random_range(-0.5..0.5)
        //        + std::f32::consts::PI / 4.;
        //    let start = Vec2::from_angle(angle) * sampler.sample(&mut rng);
        //
        //    let material = commands
        //        .spawn((Material::Shield, LinearVelocity::ZERO))
        //        .id();
        //    commands.entity(material).animation().insert_tween_here(
        //        Duration::from_secs_f32(
        //            (1.5 + rng.random_range(-dur_variation..dur_variation)) * 0.75,
        //        ),
        //        EaseKind::CubicOut,
        //        material
        //            .into_target()
        //            .with(linear_velocity(start, Vec2::ZERO)),
        //    );
        //    commands.entity(cluster).add_child(material);
        //}
    }
}

fn move_clusters(mut clusters: Query<&mut Transform, With<MaterialCluster>>, time: Res<Time>) {
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
