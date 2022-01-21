use bevy::prelude::*;

mod clipboard;

#[derive(Component)]
struct LightContainer;

#[derive(Component)]
struct CameraContainer;

#[derive(Default)]
pub struct WordleShare(String);

const CUBE_SIZE: (f32, f32, f32) = (2.15, 2.11, 2.23);

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 12.0f32,
        })
        .add_plugins(DefaultPlugins)
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
    asset_server: Res<AssetServer>,
    wordle_share: Res<WordleShare>,
) {
    if !wordle_share.is_changed() {
        return;
    }

    let valid = ['⬛', '⬜', '🟨', '🟩'];

    let mut y: f32 = 0.0;

    for line in wordle_share.0.lines().rev() {
        let mut x: f32 = -2.5 * CUBE_SIZE.0;

        if line.chars().any(|c| !valid.contains(&c)) {
            continue;
        }

        for char in line.chars() {
            let handle = match char {
                '🟨' => Some(asset_server.load("container-yellow.glb#Scene0")),
                '🟩' => Some(asset_server.load("container-green.glb#Scene0")),
                _ => None,
            };

            if let Some(handle) = handle {
                commands
                    .spawn_bundle((Transform::from_xyz(x, y, 0.0), GlobalTransform::default()))
                    .with_children(|parent| {
                        parent.spawn_scene(handle);
                    });
            }

            x += CUBE_SIZE.0;
        }

        y += CUBE_SIZE.1;
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    /*
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1000.0 })),
        material: materials.add(Color::rgb(0.3, 0.3, 0.3).into()),
        ..Default::default()
    });*/
    commands
        .spawn_bundle((
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
        ))
        .with_children(|parent| {
            parent.spawn_scene(asset_server.load("floor.glb#Scene0"));
        });

    commands
        .spawn_bundle((
            Transform::default(),
            GlobalTransform::default(),
            CameraContainer,
        ))
        .with_children(|parent| {
            parent.spawn_bundle(PerspectiveCameraBundle {
                transform: Transform::from_xyz(13.0, 13.0, 13.0)
                    .looking_at(Vec3::new(-6.0, 1.0, -6.0), Vec3::Y),
                ..Default::default()
            });
        });

    let light_dist = 8.0;
    let light_mesh = meshes.add(Mesh::from(shape::UVSphere {
        sectors: 128,
        stacks: 64,
        ..Default::default()
    }));

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
                        intensity: 1000.0,
                        shadows_enabled: true,
                        radius: 20.,
                        ..Default::default()
                    },
                    transform,
                    ..Default::default()
                });
                /*parent.spawn_bundle(PbrBundle {
                    mesh: light_mesh.clone(),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.5, 0.5, 1.0),
                        unlit: true,
                        ..Default::default()
                    }),
                    transform: transform.with_scale(Vec3::splat(1.)),
                    ..Default::default()
                });*/
            }
        });
}
