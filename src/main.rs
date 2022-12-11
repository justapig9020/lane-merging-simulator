mod vehicle;

use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy_physimple::prelude::*;
use vehicle::Destination;
use vehicle::Vehicle;

// unit:
//  distance: meter
//  time: second
const SPEED: f32 = 60.0 * 1000.0 / 3600.0;
const MAX_SPEED: f32 = 80.0 * 1000.0 / 3600.0;
const MAX_ACCELERATION: f32 = 6.0;
const VEHICLE_SIZE: f32 = 25.0;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        width: 1280.0,
        height: 720.0,
        title: "Lane Merging Simulator".to_string(),
        ..Default::default()
    });

    app.add_plugins(DefaultPlugins).add_plugin(Physics2dPlugin);

    app.add_startup_system(setup_sys);
    app.add_system(bevy::window::close_on_esc)
        .add_system(bevy::input::mouse::mouse_button_input_system)
        .add_system(move_vehicle_sys)
        .add_system(vehicle_movement_sys.after(move_vehicle_sys))
        .add_system(set_target_sys);
    app.run();
}

fn setup_sys(mut coms: Commands) {
    coms.spawn_bundle(Camera2dBundle::default());

    // Vehicle
    coms.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::splat(VEHICLE_SIZE)),
            color: Color::CYAN,
            ..Default::default()
        },
        ..Default::default()
    })
    .insert_bundle(KinematicBundle {
        shape: CollisionShape::Square(Square::size(Vec2::splat(VEHICLE_SIZE))),
        ..Default::default()
    })
    .insert(Vehicle::new(SPEED, MAX_SPEED, MAX_ACCELERATION))
    .insert(Destination(Vec2::ZERO));
}

fn vehicle_movement_sys(time: Res<Time>, mut q: Query<(&mut Vel, &Vehicle)>) {
    for (mut vel, veh) in q.iter_mut() {
        let vec = Vec2 { x: 1.0, y: 0.0 };
        vel.0 = vel
            .0
            .lerp(vec * veh.current_speed(), time.delta_seconds() * 5.0);
        vel.0 += vec * veh.current_speed() * time.delta_seconds();
    }
}

fn move_vehicle_sys(time: Res<Time>, mut q: Query<(&Vel, &mut Transform)>) {
    for (v, mut t) in q.iter_mut() {
        t.translation += v.0.extend(0.0) * time.delta_seconds();
    }
}

fn screen_to_world(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    screen_pos: Vec2,
) -> Vec2 {
    // get the size of the window
    let window_size = Vec2::new(window.width() as f32, window.height() as f32);

    // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
    let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

    // matrix for undoing the projection and camera transform
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

    // use it to convert ndc to world-space coordinates
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

    // reduce it to a 2D value
    let world_pos: Vec2 = world_pos.truncate();
    world_pos
}

fn set_target_sys(
    mut coms: Commands,
    mut mouse: EventReader<MouseButtonInput>,
    // need to get window dimensions
    windows: Res<Windows>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_dest: Query<&mut Destination>,
) {
    let (camera, camera_transform) = q_camera.single();
    for ev in mouse.iter() {
        match ev.state {
            ButtonState::Pressed if ev.button == MouseButton::Left => {
                let window = windows.get_primary().unwrap();
                if let Some(cursor) = window.cursor_position() {
                    let cursor = screen_to_world(window, camera, camera_transform, cursor);
                    println!("Spawn a bundle @ {cursor}");
                    coms.spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::splat(VEHICLE_SIZE)),
                            color: Color::CRIMSON,
                            ..Default::default()
                        },
                        transform: Transform {
                            translation: cursor.extend(0.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                    for mut dest in q_dest.iter_mut() {
                        dest.0 = cursor;
                    }
                }
            }
            _ => {}
        }
    }
}
