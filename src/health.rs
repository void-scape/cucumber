use bevy::prelude::*;
use physics::{
    CollisionSystems, Physics, PhysicsSystems,
    layers::{RegisterPhysicsLayer, TriggersWith},
    trigger::TriggerEnter,
};
use std::ops::Deref;

pub struct HealthPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct HealthSet;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.register_trigger_layer::<HitBox>()
            .configure_sets(
                Physics,
                HealthSet
                    .after(CollisionSystems::Resolution)
                    .after(PhysicsSystems::Collision),
            )
            .add_systems(
                Physics,
                (
                    update_triggered_hitboxes,
                    update_health,
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
    current: usize,
    max: usize,
    dead: bool,
}

impl Health {
    pub const PLAYER: Self = Health::full(3);
    pub const INVINCIBLE: Self = Health::full(usize::MAX);

    pub const fn full(max: usize) -> Self {
        Self {
            current: max,
            dead: false,
            max,
        }
    }

    pub fn heal(&mut self, heal: usize) {
        self.current = (self.current + heal).max(self.max);
    }

    pub fn damage(&mut self, damage: usize) {
        self.current = self.current.saturating_sub(damage);
        self.dead = self.current == 0;
    }

    pub fn damage_all(&mut self) {
        self.current = 0;
        self.dead = true;
    }

    pub fn current(&self) -> usize {
        self.current
    }

    pub fn max(&self) -> usize {
        self.max
    }

    pub fn dead(&self) -> bool {
        self.dead || self.current == 0
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

/// A trigger layer for an entity's hit box.
#[derive(Debug, Clone, Copy, Component)]
#[require(TriggersWith<HurtBox>)]
pub struct HitBox(Damage);

impl HitBox {
    pub const ONE: Self = Self::new(1);

    pub const fn new(damage: usize) -> Self {
        Self(Damage(damage))
    }

    pub fn damage(&self) -> Damage {
        self.0
    }
}

/// A trigger layer for an entity's hurt box.
#[derive(Debug, Default, Clone, Copy, Component)]
#[require(TriggeredHitBoxes)]
pub struct HurtBox;

/// Prevents the [`Damage`] collected in [`TriggeredHitBoxes`] from being applied to an entity's
/// [`Health`].
///
/// [`TriggeredHitBoxes`] is updated in the [`Physics`] schedule, so look either read from it after
/// [`update_triggered_hitboxes`] or when it is [`Changed`].
#[derive(Default, Component)]
pub struct ManualHurtBox;

/// Contains the entities and their corresponding [`HurtBox`] [`Damage`].
///
/// Updated during the [`CollisionSystems::Resolution`] system set.
#[derive(Default, Component)]
pub struct TriggeredHitBoxes(Vec<(Entity, Damage)>);

impl TriggeredHitBoxes {
    pub fn triggered(&self) -> &[(Entity, Damage)] {
        &self.0
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter().map(|(e, _)| e)
    }
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Damage(usize);

impl Deref for Damage {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Damage {
    pub fn new(damage: usize) -> Self {
        Self(damage)
    }

    pub fn damage(&self) -> usize {
        self.0
    }
}

pub fn update_triggered_hitboxes(
    mut hurtbox_query: Query<&mut TriggeredHitBoxes, (With<HurtBox>, With<TriggersWith<HitBox>>)>,
    mut reader: EventReader<TriggerEnter>,
    hitbox_query: Query<&HitBox>,
) {
    for mut cache in hurtbox_query.iter_mut() {
        cache.0.clear();
    }

    for event in reader.read() {
        if let Ok(mut cache) = hurtbox_query.get_mut(event.target) {
            let Ok(damage) = hitbox_query.get(event.trigger).map(|h| h.damage()) else {
                continue;
            };

            cache.0.push((event.trigger, damage));
        }
    }
}

pub fn update_health(
    mut health_query: Query<(&mut Health, &TriggeredHitBoxes), Without<ManualHurtBox>>,
) {
    for (mut health, hit_boxes) in health_query.iter_mut() {
        for (_, damage) in hit_boxes.triggered().iter() {
            health.damage(damage.damage());
        }
    }
}

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
        commands.entity(entity).despawn_recursive();
    }
}
