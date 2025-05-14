use super::{BuckShot, CrissCross, MineThrower, OrbSlinger, Swarm, timeline::WaveTimeline};
use crate::bullet::emitter::EmitterDelay;
use crate::{Avian, DespawnRestart, GameState, HEIGHT, boss::gradius};
use avian2d::prelude::Physics;
use bevy::prelude::*;

const DEFAULT_FORMATION_VEL: Vec2 = Vec2::new(0., -8.);
const NUM_SWARM: usize = 10;
const ENEMY_Z: f32 = 0.;

pub struct FormationPlugin;

impl Plugin for FormationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_formations, despawn_formations)
                .in_set(FormationSet)
                .chain()
                .run_if(in_state(GameState::Game)),
        )
        .add_systems(Avian, update_formations);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct FormationSet;

// We leak formations when they die until the game restarts, but this is fine
#[derive(Debug, Clone, Component)]
#[require(Transform, Visibility, DespawnRestart)]
pub struct Formation {
    spawn: fn(&mut EntityCommands, &AssetServer),
    velocity: Vec2,
}

impl Formation {
    pub fn new(spawn: fn(&mut EntityCommands, &AssetServer)) -> Self {
        Self::with_velocity(DEFAULT_FORMATION_VEL, spawn)
    }

    pub fn with_velocity(velocity: Vec2, spawn: fn(&mut EntityCommands, &AssetServer)) -> Self {
        Self { spawn, velocity }
    }
}

pub fn mine_thrower() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![MineThrower]);
    })
}

pub fn quad_mine_thrower() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![
            (MineThrower, Transform::from_xyz(-20., 0., 0.)),
            (MineThrower, Transform::from_xyz(20., 0., 1.)),
            (MineThrower, Transform::from_xyz(-40., 10., 2.)),
            (MineThrower, Transform::from_xyz(40., 10., 3.)),
        ]);
    })
}

pub fn double_buck_shot() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![
            (BuckShot, Transform::from_xyz(-30., 0., 0.)),
            (BuckShot, Transform::from_xyz(30., 0., 0.)),
        ]);
    })
}

pub fn orb_slinger() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![OrbSlinger]);
    })
}

pub fn double_orb_slinger() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![
            (OrbSlinger, Transform::from_xyz(-40., 0., 0.)),
            (OrbSlinger, Transform::from_xyz(40., 0., 0.)),
        ]);
    })
}

pub fn crisscross() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![CrissCross]);
    })
}

pub fn double_crisscross() -> Formation {
    Formation::new(|formation: &mut EntityCommands, _| {
        formation.insert(children![
            (CrissCross, Transform::from_xyz(-40., 0., 0.)),
            (CrissCross, Transform::from_xyz(40., 0., 0.)),
        ]);
    })
}

const SWARM_OFFSET: f32 = crate::WIDTH / 1.2;

pub fn swarm() -> Formation {
    Formation::with_velocity(
        DEFAULT_FORMATION_VEL * 1.5,
        |formation: &mut EntityCommands, _| {
            formation.with_children(|root| {
                for i in 0..NUM_SWARM {
                    let x = (i as f32 - 5.) * 10.;
                    root.spawn((
                        Swarm,
                        EmitterDelay::new(0.2 * i as f32),
                        Transform::from_xyz(x + SWARM_OFFSET, 0., 0.),
                    ));
                }

                for i in 0..NUM_SWARM {
                    let x = (i as f32 - 5.) * 10.;
                    let y = 10.;
                    root.spawn((
                        Swarm,
                        EmitterDelay::new(0.2 * i as f32),
                        Transform::from_xyz(x + SWARM_OFFSET, y, 0.),
                    ));
                }
            });
        },
    )
}

pub fn boss() -> Formation {
    Formation::with_velocity(Vec2::ZERO, |formation: &mut EntityCommands, _| {
        formation.commands().remove_resource::<WaveTimeline>();
        formation.commands().spawn(gradius::Gradius);
    })
}

//#[derive(Component)]
//#[relationship(relationship_target = Units)]
//#[component(on_remove = on_remove_platoon)]
//struct Platoon(Entity);
//
//fn on_remove_platoon(mut world: DeferredWorld, ctx: HookContext) {
//    let platoon = world.entity(ctx.entity).get::<Platoon>().unwrap().0;
//    let position = world
//        .entity(ctx.entity)
//        .get::<GlobalTransform>()
//        .unwrap()
//        .compute_transform()
//        .translation
//        .xy();
//
//    world
//        .commands()
//        .entity(platoon)
//        .entry::<UnitDeaths>()
//        .and_modify(move |mut deaths| deaths.death_position(position));
//}
//
//#[derive(Component)]
//#[relationship_target(relationship = Platoon)]
//#[require(UnitDeaths)]
//struct Units(Vec<Entity>);
//
//#[derive(Default, Component)]
//struct UnitDeaths(Vec<Vec2>);
//
//impl UnitDeaths {
//    pub fn death_position(&mut self, position: Vec2) {
//        self.0.push(position);
//    }
//
//    pub fn last_death_position(&self) -> Option<Vec2> {
//        self.0.last().copied()
//    }
//}

const LARGEST_SPRITE_SIZE: f32 = 16.;
const PADDING: f32 = LARGEST_SPRITE_SIZE;
const FORMATION_EASE_DUR: f32 = 2.;

fn spawn_formations(
    mut commands: Commands,
    server: Res<AssetServer>,
    new_formations: Query<(Entity, &Formation), Added<Formation>>,
    //formations: Query<(Entity, &Transform), (With<Formation>, With<Units>)>,
) {
    //let mut additional_height = 0.;
    for (root, formation) in new_formations.iter() {
        info!("spawn new formation");

        //additional_height += formation.height() + PADDING;

        let start_y = HEIGHT / 2. + LARGEST_SPRITE_SIZE / 2.;
        //let end_y = HEIGHT / 2. + formation.topy() - LARGEST_SPRITE_SIZE / 2.;

        let start = Vec3::new(0., start_y, ENEMY_Z);
        //let end = Vec3::new(0., end_y - 20., 0.);

        //commands.entity(root).animation().insert(tween(
        //    Duration::from_secs_f32(FORMATION_EASE_DUR),
        //    EaseKind::SineOut,
        //    root.into_target().with(translation(start, end)),
        //));

        let mut commands = commands.entity(root);
        (formation.spawn)(&mut commands, &server);
        commands.insert(Transform::from_translation(start));
    }

    //if additional_height > 0. {
    //    for (entity, transform) in formations.iter() {
    //        commands.animation().insert(tween(
    //            Duration::from_secs_f32(FORMATION_EASE_DUR),
    //            EaseKind::SineOut,
    //            entity.into_target().with(translation(
    //                transform.translation,
    //                Vec3::new(0., additional_height, 0.),
    //            )),
    //        ));
    //    }
    //}
}

fn update_formations(
    time: Res<Time<Physics>>,
    mut formations: Query<(&mut Transform, &Formation)>,
) {
    for (mut transform, formation) in formations.iter_mut() {
        transform.translation += formation.velocity.extend(0.) * time.delta_secs();
    }
}

//fn despawn_off_screen(
//    mut commands: Commands,
//    enemies: Query<(Entity, &GlobalTransform), With<EnemyType>>,
//) {
//    for (entity, transform) in enemies.iter() {
//        let p = transform.compute_transform().translation;
//        let w = crate::WIDTH / 2.;
//        let h = crate::HEIGHT / 2.;
//
//        if p.x < -w || p.y > w || p.y < -h || p.y > h {
//            commands.entity(entity).despawn();
//        }
//    }
//}

fn despawn_formations(
    mut commands: Commands,
    formations: Query<(Entity, &Formation), Without<Children>>,
    //formations: Query<(Entity, &Formation, &UnitDeaths), Without<Units>>,
    //off_screen: Query<(Entity, &Transform, &Formation)>,
) {
    for (entity, _formation) in formations.iter() {
        info!("despawn dead formation");
        commands.entity(entity).despawn();

        //if formation.drop_pickup_heuristic() {
        //    let mut commands = commands.spawn_empty();
        //    let position = deaths.last_death_position().unwrap();
        //
        //    pickups::spawn_random_pickup(
        //        &mut commands,
        //        (
        //            Transform::from_translation(position.extend(0.)),
        //            pickups::velocity(),
        //        ),
        //    );
        //}
    }

    //for (entity, transform, formation) in off_screen.iter() {
    //    let p = transform.translation;
    //    let w = crate::WIDTH / 2.;
    //    let h = crate::HEIGHT / 2. + formation.height() + 10.;
    //
    //    if p.x < -w || p.x > w || p.y < -h || p.y > h {
    //        info!("despawn formation off screen");
    //        commands
    //            .entity(entity)
    //            .despawn_related::<Children>()
    //            .despawn();
    //    }
    //}
}
