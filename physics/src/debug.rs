use super::collision::{Collider, CollidesWith, DynamicBody, StaticBody};
use super::trigger::{CollisionTrigger, TriggerEvent};
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::sprite::{Wireframe2d, Wireframe2dColor};
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;

#[derive(Resource)]
pub struct ShowCollision(pub bool);

impl Collider {
    fn debug_wireframe_bundle(
        &self,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> impl Bundle {
        (
            match self {
                Self::Rect(rect) => (
                    Mesh2d(meshes.add(Rectangle::new(rect.size.x, rect.size.y))),
                    MeshMaterial2d(materials.add(Color::NONE)),
                    Transform::from_xyz(
                        rect.tl.x + rect.size.x / 2.,
                        rect.tl.y - rect.size.y / 2.,
                        999.,
                    ),
                ),
                Self::Circle(circle) => (
                    Mesh2d(meshes.add(Circle::new(circle.radius))),
                    MeshMaterial2d(materials.add(Color::NONE)),
                    Transform::from_xyz(circle.position.x, circle.position.y, 999.),
                ),
            },
            HIGH_RES_LAYER,
            Wireframe2d,
            Wireframe2dColor {
                color: Srgba::WHITE.into(),
            },
        )
    }
}

#[derive(Component)]
pub struct DebugWireframe;

#[derive(Component)]
pub struct Marked;

pub fn update_show_collision(
    mut reader: EventReader<KeyboardInput>,
    mut show: ResMut<ShowCollision>,
) {
    for event in reader.read() {
        if matches!(
            event,
            KeyboardInput {
                key_code: KeyCode::KeyP,
                state: ButtonState::Pressed,
                repeat: false,
                ..
            }
        ) {
            show.0 = !show.0;
        }
    }
}

pub fn debug_display_collider_wireframe(
    naked_colliders: Query<(Entity, &Collider), (Without<Marked>, With<Transform>)>,
    naked_trigger_colliders: Query<(Entity, &CollisionTrigger), (Without<Marked>, With<Transform>)>,
    frames: Query<Entity, With<DebugWireframe>>,
    marked: Query<Entity, With<Marked>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    show: Res<ShowCollision>,
) {
    if show.0 {
        for (entity, collider) in naked_colliders
            .iter()
            .chain(naked_trigger_colliders.iter().map(|(e, t)| (e, &t.0)))
        {
            let wireframe = commands
                .spawn((
                    DebugWireframe,
                    collider.debug_wireframe_bundle(&mut meshes, &mut materials),
                ))
                .id();
            commands
                .entity(entity)
                .insert((Visibility::Visible, Marked))
                .add_child(wireframe);
        }
    } else {
        for entity in frames.iter() {
            commands.entity(entity).despawn();
        }

        for entity in marked.iter() {
            commands.entity(entity).remove::<Marked>();
        }
    }
}

pub fn debug_show_trigger_color(
    triggers: Query<&Children, (With<CollisionTrigger>, With<Marked>)>,
    mut wireframes: Query<&mut Wireframe2dColor>,
    mut reader: EventReader<TriggerEvent>,
) {
    for event in reader.read() {
        if let Ok(children) = triggers.get(event.trigger) {
            for child in children.iter() {
                if let Ok(mut frame) = wireframes.get_mut(child) {
                    frame.color = Srgba::GREEN.into();
                }
            }
        }
    }
}

pub fn debug_show_collision_color(
    static_bodies: Query<(&GlobalTransform, &Collider, &Children), With<StaticBody>>,
    dynamic_bodies: Query<(&GlobalTransform, &Collider), With<DynamicBody>>,
    mut wireframes: Query<&mut Wireframe2dColor>,
) {
    for mut frame in wireframes.iter_mut() {
        frame.color = Srgba::WHITE.into();
    }

    for (dyn_t, dyn_c) in dynamic_bodies.iter() {
        let dyn_c = dyn_c.absolute(&dyn_t.compute_transform());
        for (t, c, children) in static_bodies.iter() {
            if dyn_c.collides_with(&c.absolute(&t.compute_transform())) {
                for child in children.iter() {
                    if let Ok(mut frame) = wireframes.get_mut(child) {
                        frame.color = Srgba::RED.into();
                    }
                }
            }
        }
    }
}
