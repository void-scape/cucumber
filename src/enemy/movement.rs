use crate::GameState;
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (update_back_and_forth, update_circle, update_figure8)
                .run_if(in_state(GameState::Game)),
        );
    }
}

#[derive(Component)]
#[require(Angle)]
pub struct BackAndForth {
    pub radius: f32,
    pub speed: f32,
}

fn update_back_and_forth(
    mut commands: Commands,
    init_query: Query<(Entity, &Transform), (With<BackAndForth>, Without<Center>)>,
    mut query: Query<(&BackAndForth, &Center, &mut Angle, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, transform) in init_query.iter() {
        commands
            .entity(entity)
            .insert(Center(transform.translation.xy()));
    }

    for (baf, center, mut angle, mut transform) in query.iter_mut() {
        transform.translation.x = center.0.x + baf.radius * angle.0.cos();
        angle.0 += baf.speed * time.delta_secs();
        if angle.0 >= std::f32::consts::PI * 2. {
            angle.0 = 0.;
        }
    }
}

#[derive(Component)]
#[require(Angle)]
pub struct Circle {
    pub radius: f32,
    pub speed: f32,
}

#[derive(Default, Component)]
pub struct Angle(pub f32);

#[derive(Component)]
pub struct Center(pub Vec2);

fn update_circle(
    mut commands: Commands,
    init_query: Query<(Entity, &Transform), (With<Circle>, Without<Center>)>,
    mut query: Query<(&Circle, &Center, &mut Angle, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, transform) in init_query.iter() {
        commands
            .entity(entity)
            .insert(Center(transform.translation.xy()));
    }

    for (circle, center, mut angle, mut transform) in query.iter_mut() {
        transform.translation.x = center.0.x + circle.radius * angle.0.cos();
        transform.translation.y = center.0.y + circle.radius * angle.0.sin();
        angle.0 += circle.speed * time.delta_secs();
        if angle.0 >= std::f32::consts::PI * 2. {
            angle.0 = 0.;
        }
    }
}

#[derive(Component)]
#[require(Angle)]
pub struct Figure8 {
    pub radius: f32,
    pub speed: f32,
}

fn update_figure8(
    mut commands: Commands,
    init_query: Query<(Entity, &Transform), (With<Figure8>, Without<Center>)>,
    mut query: Query<(&mut Figure8, &Center, &mut Angle, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, transform) in init_query.iter() {
        commands
            .entity(entity)
            .insert(Center(transform.translation.xy()));
    }

    for (figure8, center, mut angle, mut transform) in query.iter_mut() {
        let t = angle.0;
        transform.translation.x = center.0.x + figure8.radius * t.sin();
        transform.translation.y = center.0.y + figure8.radius * t.sin() * t.cos();

        angle.0 += figure8.speed * time.delta_secs();
        if angle.0 >= std::f32::consts::TAU {
            angle.0 = 0.;
        }
    }
}
