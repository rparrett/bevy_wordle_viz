use bevy::prelude::*;
use bevy_easings::*;
use rand::prelude::*;
use std::str::FromStr;
use wordle::{WordleGrid, WordleGuessKind};

#[cfg_attr(not(target_arch = "wasm32"), path = "native_clipboard.rs")]
#[cfg_attr(target_arch = "wasm32", path = "wasm_clipboard.rs")]
mod clipboard;
mod wordle;

#[derive(Component)]
struct LightContainer;

#[derive(Component)]
struct CameraContainer;
#[derive(Component)]
pub struct WordleBox;
#[derive(Component)]
pub struct StartTime(f64);
#[derive(Component)]
pub struct Destination(Transform);

#[derive(Default)]
pub struct WordleShare(String);

pub struct Handles {
    green_box: Handle<Scene>,
    yellow_box: Handle<Scene>,
    black_box: Handle<Scene>,
    floor: Handle<Scene>,
}
impl FromWorld for Handles {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            green_box: asset_server.load("container-green.glb#Scene0"),
            yellow_box: asset_server.load("container-yellow.glb#Scene0"),
            black_box: asset_server.load("container-black.glb#Scene0"),
            floor: asset_server.load("floor.glb#Scene0"),
        }
    }
}

const CUBE_SIZE: (f32, f32, f32) = (2.15, 2.11, 2.23);

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy Wordle Viz".to_string(),
            ..Default::default()
        })
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 16.0f32,
        })
        .add_plugins(DefaultPlugins)
        .init_resource::<Handles>()
        .init_resource::<WordleShare>()
        .add_plugin(clipboard::ClipboardPlugin)
        .add_plugin(EasingsPlugin)
        .add_startup_system(setup)
        .add_system(spawn_wordle)
        .add_system(rotate_lights)
        .add_system(rotate_camera)
        .add_system(drop_boxes)
        .run();
}

fn rotate_lights(time: Res<Time>, mut query: Query<&mut Transform, With<LightContainer>>) {
    for mut transform in query.iter_mut() {
        transform.rotation = Quat::from_rotation_y((time.seconds_since_startup() * 0.2) as f32);
    }
}

fn rotate_camera(time: Res<Time>, mut query: Query<&mut Transform, With<CameraContainer>>) {
    for mut transform in query.iter_mut() {
        transform.rotation =
            Quat::from_rotation_y(((time.seconds_since_startup() * -0.2).sin() * 0.1) as f32);
    }
}

fn drop_boxes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &Transform, &Destination, &StartTime)>,
) {
    let current_time = time.seconds_since_startup();
    for (entity, transform, destination, start_time) in query.iter_mut() {
        if start_time.0 > current_time {
            continue;
        }

        commands.entity(entity).insert(transform.ease_to(
            destination.0,
            EaseFunction::CubicIn,
            EasingType::Once {
                duration: std::time::Duration::from_secs(2),
            },
        ));
        commands.entity(entity).remove::<Destination>();
        commands.entity(entity).remove::<StartTime>();
    }
}

fn spawn_wordle(
    mut commands: Commands,
    handles: Res<Handles>,
    wordle_share: Res<WordleShare>,
    query: Query<Entity, With<WordleBox>>,
    time: Res<Time>,
) {
    if !wordle_share.is_changed() {
        return;
    }

    let grid = WordleGrid::from_str(&wordle_share.0);

    let grid = match grid {
        Ok(grid) => grid,
        _ => return,
    };

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let left = -2.0 * CUBE_SIZE.0;

    let mut delay = 0.;

    let delay_step = 0.4;
    let row_delay_step = 0.6;
    let mut prev_row = 0;

    let current_time = time.seconds_since_startup();

    let mut rng = thread_rng();

    for (row, col, guess) in grid.snake_iter() {
        let handle = match guess.kind {
            WordleGuessKind::InWord => handles.yellow_box.clone(),
            WordleGuessKind::Correct => handles.green_box.clone(),
            WordleGuessKind::NotInWord if guess.support || guess.topper => {
                handles.black_box.clone()
            }
            _ => continue,
        };

        delay += if row != prev_row {
            row_delay_step
        } else {
            delay_step
        };

        prev_row = row;

        let x = left + CUBE_SIZE.0 * col as f32;
        let y = CUBE_SIZE.1 * row as f32;

        let destination = Vec3::new(x, y, 0.0);
        let rotation =
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 * rng.gen_range(0..=1) as f32);
        let transform = Transform::from_translation(destination + Vec3::new(0.0, 14.5, 0.0))
            .with_rotation(rotation);

        commands
            .spawn_bundle((
                transform,
                GlobalTransform::default(),
                WordleBox,
                StartTime(current_time + delay),
                Destination(Transform::from_translation(destination).with_rotation(rotation)),
            ))
            .with_children(|parent| {
                parent.spawn_scene(handle);
            });
    }
}

fn setup(mut commands: Commands, handles: Res<Handles>) {
    commands
        .spawn_bundle((
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(handles.floor.clone());
        });

    commands
        .spawn_bundle((
            Transform::default(),
            GlobalTransform::default(),
            CameraContainer,
        ))
        .with_children(|parent| {
            parent.spawn_bundle(PerspectiveCameraBundle {
                transform: Transform::from_xyz(14.0, 15.0, 14.0)
                    .looking_at(Vec3::new(-7.0, 1.0, -7.0), Vec3::Y),
                ..Default::default()
            });
        });

    let light_dist = 8.0;

    commands
        .spawn_bundle((
            Transform::default(),
            GlobalTransform::default(),
            LightContainer,
        ))
        .with_children(|parent| {
            for transform in [
                Transform::from_xyz(light_dist, 8.0, light_dist),
                Transform::from_xyz(-light_dist, 8.0, -light_dist),
                Transform::from_xyz(-light_dist, 8.0, light_dist),
                Transform::from_xyz(light_dist, 8.0, -light_dist),
            ] {
                parent.spawn_bundle(PointLightBundle {
                    point_light: PointLight {
                        intensity: 900.0,
                        shadows_enabled: true,
                        radius: 20.,
                        ..Default::default()
                    },
                    transform,
                    ..Default::default()
                });
            }
        });
}
