use bevy::prelude::*;
use bevy_sequence::prelude::*;
use bevy_tween::bevy_time_runner::TimeRunner;
use bevy_tween::combinator::AnimationCommands;
use bevy_tween::prelude::*;
use rand::Rng;

pub struct SequenceTweenPlugin;

impl Plugin for SequenceTweenPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FragmentEvent<TweenAnimation>>()
            .add_systems(Update, (insert_timeouts, emit_tween_timeouts));
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
