use bevy::prelude::*;
use std::collections::HashSet;

#[cfg_attr(not(target_arch = "wasm32"), path = "native_clipboard.rs")]
#[cfg_attr(target_arch = "wasm32", path = "wasm_clipboard.rs")]
mod clipboard;

#[derive(Component)]
struct LightContainer;

#[derive(Component)]
struct CameraContainer;
#[derive(Component)]
pub struct WordleBox;

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
        .add_startup_system(setup)
        .add_system(spawn_wordle)
        .add_system(rotate_lights)
        .add_system(rotate_camera)
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

fn spawn_wordle(
    mut commands: Commands,
    handles: Res<Handles>,
    wordle_share: Res<WordleShare>,
    query: Query<Entity, With<WordleBox>>,
) {
    if !wordle_share.is_changed() {
        return;
    }

    let valid = ['⬛', '⬜', '🟨', '🟩'];

    let grid = wordle_share
        .0
        .lines()
        .filter(|line| line.chars().all(|c| valid.contains(&c)))
        .collect::<Vec<_>>();

    if grid.is_empty() {
        return;
    }

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let mut needs_support = HashSet::new();

    let top = (grid.len() - 1) as f32 * CUBE_SIZE.1;
    let left = -2.0 * CUBE_SIZE.0;

    for (row, chars) in grid.iter().enumerate() {
        for (col, char) in chars.chars().enumerate() {
            if char == '🟨' || char == '🟩' {
                needs_support.insert(col);
            }

            let handle = match char {
                '🟨' => handles.yellow_box.clone(),
                '🟩' => handles.green_box.clone(),
                '⬛' | '⬜' if needs_support.contains(&col) => handles.black_box.clone(),
                _ => continue,
            };

            let x = left + CUBE_SIZE.0 * col as f32;
            let y = top - CUBE_SIZE.1 * row as f32;

            commands
                .spawn_bundle((
                    Transform::from_xyz(x, y, 0.0),
                    GlobalTransform::default(),
                    WordleBox,
                ))
                .with_children(|parent| {
                    parent.spawn_scene(handle);
                });
        }
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
