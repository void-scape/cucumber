use avian2d::prelude::{Physics, PhysicsTime};
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy_sequence::prelude::*;
use bevy_tween::bevy_time_runner::TimeRunner;
use bevy_tween::combinator::AnimationCommands;
use bevy_tween::prelude::*;
use bevy_tween::tween::apply_resource_tween_system;
use rand::Rng;

pub struct TweenPlugin;

impl Plugin for TweenPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FragmentEvent<TweenAnimation>>()
            .insert_resource(PhysicsTimeMult::default())
            .add_tween_systems(apply_resource_tween_system::<PhysicsTimeTween>)
            .add_systems(
                Update,
                (
                    insert_timeouts,
                    emit_tween_timeouts,
                    run_tween_on_end,
                    update_physics_time,
                ),
            );
    }
}

#[derive(Resource)]
pub struct PhysicsTimeMult(pub f32);

impl Default for PhysicsTimeMult {
    fn default() -> Self {
        Self(1.)
    }
}

pub fn physics_time_mult(start: f32, end: f32) -> PhysicsTimeTween {
    PhysicsTimeTween::new(start, end)
}

#[derive(Component)]
pub struct PhysicsTimeTween {
    start: f32,
    end: f32,
}

impl PhysicsTimeTween {
    pub fn new(start: f32, end: f32) -> Self {
        Self { start, end }
    }
}

impl Interpolator for PhysicsTimeTween {
    type Item = PhysicsTimeMult;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        item.0 = self.start.lerp(self.end, value);
    }
}

fn update_physics_time(mut time: ResMut<Time<Physics>>, mult: Res<PhysicsTimeMult>) {
    if mult.is_changed() {
        time.set_relative_speed(mult.0);
    }
}

pub struct Tween<T>(pub T);

#[derive(Debug, Clone)]
pub struct TweenAnimation(u64);

#[derive(Component)]
struct TweenTimeout(FragmentEndEvent);

#[derive(Component)]
struct TweenKey(u64);

impl<T> IntoFragment<TweenAnimation, ()> for Tween<T>
where
    T: FnOnce(&mut AnimationCommands, &mut Duration),
{
    fn into_fragment(self, context: &Context<()>, commands: &mut Commands) -> FragmentId {
        let key = rand::rng().random();
        commands.animation().insert(self.0).insert(TweenKey(key));
        <_ as IntoFragment<TweenAnimation, ()>>::into_fragment(
            bevy_sequence::fragment::DataLeaf::new(TweenAnimation(key)),
            context,
            commands,
        )
    }
}

fn insert_timeouts(
    mut commands: Commands,
    mut reader: EventReader<FragmentEvent<TweenAnimation>>,
    tweens: Query<(Entity, &TweenKey), Without<TweenTimeout>>,
) {
    for event in reader.read() {
        if let Some((entity, _)) = tweens.iter().find(|(_, key)| key.0 == event.data.0) {
            commands.entity(entity).insert(TweenTimeout(event.end()));
        }
    }
}

fn emit_tween_timeouts(
    mut commands: Commands,
    mut writer: EventWriter<FragmentEndEvent>,
    tweens: Query<(Entity, &TimeRunner, &TweenTimeout)>,
) {
    for (entity, runner, timeout) in tweens.iter() {
        if runner.is_completed() {
            writer.write(timeout.0);
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
pub struct OnEnd(SystemId);

impl OnEnd {
    pub fn new<Marker>(
        commands: &mut Commands,
        system: impl IntoSystem<(), (), Marker> + 'static,
    ) -> Self {
        Self(commands.register_system(system))
    }
}

fn run_tween_on_end(mut commands: Commands, tweens: Query<(Entity, &TimeRunner, &OnEnd)>) {
    for (tween, runner, on_end) in tweens.iter() {
        if runner.is_completed() {
            commands.run_system(on_end.0);
            commands.unregister_system(on_end.0);
            commands.entity(tween).despawn();
        }
    }
}

//#[derive(Event)]
//pub struct TweenFinished<Data = Unit> {
//    pub tween: Entity,
//    pub data: Data,
//}
//
//#[derive(Component)]
//#[component(on_add = Self::insert_data)]
//pub struct EmitFinished<Data = Unit>(pub Data);
//
//impl<Data> EmitFinished<Data>
//where
//    Data: Clone + Send + Sync + 'static,
//{
//    fn insert_data(mut world: DeferredWorld, ctx: HookContext) {
//        let data = world
//            .get::<EmitFinished<Data>>(ctx.entity)
//            .unwrap()
//            .0
//            .clone();
//        world
//            .commands()
//            .entity(ctx.entity)
//            .insert((TimeSpanProgress::default(), TweenEventData(data)));
//    }
//}
//
//#[derive(Clone, Copy)]
//pub struct Unit;
