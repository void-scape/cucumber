use crate::Avian;
use avian2d::prelude::*;
use bevy::prelude::*;
use std::ops::Deref;

pub struct HealthPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct HealthSet;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .configure_sets(Avian, HealthSet.after(PhysicsSet::Sync))
            .add_systems(
                Avian,
                (handle_damage, insert_dead, despawn_dead)
                    .chain()
                    .in_set(HealthSet),
            );
    }
}

#[derive(Component)]
pub struct Invincible;

#[derive(Debug, Clone, Copy, Component)]
pub struct Health {
    current: f32,
    max: f32,
    dead: bool,
}

impl Health {
    pub const fn full(max: f32) -> Self {
        Self {
            current: max,
            dead: false,
            max,
        }
    }

    pub fn heal(&mut self, heal: f32) {
        self.current = (self.current + heal).min(self.max);
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

    pub fn is_full(&self) -> bool {
        self.current == self.max
    }

    /// Calculate the current proportion of health
    /// relative to full.
    pub fn proportion(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Shield {
    current: f32,
    max: f32,
    empty: bool,
}

impl Shield {
    pub const fn full(max: f32) -> Self {
        Self {
            current: max,
            empty: false,
            max,
        }
    }

    pub fn heal(&mut self, heal: f32) {
        self.current = (self.current + heal).min(self.max);
    }

    pub fn damage(&mut self, damage: f32) {
        self.current = (self.current - damage).max(0.0);
        self.empty = self.current == 0.0;
    }

    pub fn damage_all(&mut self) {
        self.current = 0.0;
        self.empty = true;
    }

    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn empty(&self) -> bool {
        self.empty || self.current == 0.0
    }

    /// Calculate the current proportion of health
    /// relative to full.
    pub fn proportion(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}

/// Entity's [`Shield`] has reached 0.
#[derive(Default, Component)]
pub struct NoShield;

/// Entity's [`Health`] has reached 0.
#[derive(Default, Component)]
pub struct Dead;

/// Despawn an entity with the [`Dead`] marker.
#[derive(Default, Component)]
pub struct DespawnDead;

#[derive(Debug, Clone, Copy, Event)]
pub struct DamageEvent {
    pub entity: Entity,
    pub damage: f32,
}

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

pub fn handle_damage(
    mut healths: Query<(Option<&mut Shield>, &mut Health), Without<Invincible>>,
    mut reader: EventReader<DamageEvent>,
) {
    for event in reader.read() {
        if let Ok((shield, mut health)) = healths.get_mut(event.entity) {
            if let Some(mut shield) = shield {
                if shield.current() < event.damage {
                    let remaining = shield.current();
                    if !shield.empty() {
                        shield.damage_all();
                    }
                    health.damage(event.damage - remaining);
                } else {
                    shield.damage(event.damage);
                }
            } else {
                health.damage(event.damage);
            }
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
        commands.entity(entity).despawn();
    }
}
