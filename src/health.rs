use crate::Avian;
use avian2d::prelude::*;
use bevy::prelude::*;
use std::ops::Deref;

pub struct HealthPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct HealthSet;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_event::<DamageEvent>()
            .configure_sets(Avian, HealthSet.after(PhysicsSet::Sync))
            .add_systems(
                Avian,
                (
                    //update_triggered_hitboxes,
                    //update_health,
                    insert_dead,
                    despawn_dead,
                )
                    .chain()
                    .in_set(HealthSet),
            );
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Health {
    current: f32,
    max: f32,
    dead: bool,
}

impl Health {
    pub const PLAYER: Self = Health::full(3.0);
    pub const INVINCIBLE: Self = Health::full(f32::MAX);

    pub const fn full(max: f32) -> Self {
        Self {
            current: max,
            dead: false,
            max,
        }
    }

    pub fn heal(&mut self, heal: f32) {
        self.current = (self.current + heal).max(self.max);
    }

    pub fn damage(&mut self, damage: f32) {
        self.current = (self.current - damage).max(0.0);
        self.dead = self.current == 0.0;
    }

    pub fn damage_all(&mut self) {
        self.current = 0.0;
        self.dead = true;
    }

    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn dead(&self) -> bool {
        self.dead || self.current == 0.0
    }

    /// Calculate the current proportion of health
    /// relative to full.
    pub fn proportion(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}

/// Entity's [`Health`] has reached 0.
#[derive(Default, Component)]
pub struct Dead;

/// Despawn an entity with the [`Dead`] marker.
#[derive(Default, Component)]
pub struct DespawnDead;

///// A trigger layer for an entity's hit box.
//#[derive(Debug, Clone, Copy, Component)]
//#[require(TriggersWith<HurtBox>)]
//pub struct HitBox(Damage);
//
//impl HitBox {
//    pub const ONE: Self = Self::new(1);
//
//    pub const fn new(damage: usize) -> Self {
//        Self(Damage(damage))
//    }
//
//    pub fn damage(&self) -> Damage {
//        self.0
//    }
//}
//
///// A trigger layer for an entity's hurt box.
//#[derive(Debug, Default, Clone, Copy, Component)]
//#[require(TriggeredHitBoxes)]
//pub struct HurtBox;
//
///// Prevents the [`Damage`] collected in [`TriggeredHitBoxes`] from being applied to an entity's
///// [`Health`].
/////
///// [`TriggeredHitBoxes`] is updated in the [`Physics`] schedule, so look either read from it after
///// [`update_triggered_hitboxes`] or when it is [`Changed`].
//#[derive(Default, Component)]
//pub struct ManualHurtBox;
//
///// Contains the entities and their corresponding [`HurtBox`] [`Damage`].
/////
///// Updated during the [`CollisionSystems::Resolution`] system set.
//#[derive(Default, Component)]
//pub struct TriggeredHitBoxes(Vec<(Entity, Damage)>);
//
//impl TriggeredHitBoxes {
//    pub fn triggered(&self) -> &[(Entity, Damage)] {
//        &self.0
//    }
//
//    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
//        self.0.iter().map(|(e, _)| e)
//    }
//}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Damage(f32);

impl Deref for Damage {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Damage {
    pub fn new(damage: f32) -> Self {
        Self(damage)
    }

    pub fn damage(&self) -> f32 {
        self.0
    }
}

//pub fn update_triggered_hitboxes(
//    mut hurtbox_query: Query<&mut TriggeredHitBoxes, (With<HurtBox>, With<TriggersWith<HitBox>>)>,
//    mut reader: EventReader<TriggerEnter>,
//    hitbox_query: Query<&HitBox>,
//) {
//    for mut cache in hurtbox_query.iter_mut() {
//        cache.0.clear();
//    }
//
//    for event in reader.read() {
//        if let Ok(mut cache) = hurtbox_query.get_mut(event.target) {
//            let Ok(damage) = hitbox_query.get(event.trigger).map(|h| h.damage()) else {
//                continue;
//            };
//
//            cache.0.push((event.trigger, damage));
//        }
//    }
//}
//
//#[derive(Event)]
//pub struct DamageEvent {
//    pub entity: Entity,
//    pub damage: usize,
//    pub killed: bool,
//}
//
//pub fn update_health(
//    mut health_query: Query<(Entity, &mut Health, &TriggeredHitBoxes), Without<ManualHurtBox>>,
//    mut writer: EventWriter<DamageEvent>,
//) {
//    for (entity, mut health, hit_boxes) in health_query.iter_mut() {
//        for (_, damage) in hit_boxes.triggered().iter() {
//            let damage = damage.damage();
//            health.damage(damage);
//            writer.write(DamageEvent {
//                entity,
//                damage,
//                killed: health.dead(),
//            });
//        }
//    }
//}

pub fn insert_dead(mut commands: Commands, health_query: Query<(Entity, &Health), Without<Dead>>) {
    for (entity, health) in health_query.iter() {
        if health.dead() {
            commands.entity(entity).insert(Dead);
        }
    }
}

pub fn despawn_dead(
    mut commands: Commands,
    dead_query: Query<Entity, (With<Dead>, With<DespawnDead>)>,
) {
    for entity in dead_query.iter() {
        commands.entity(entity).despawn();
    }
}
