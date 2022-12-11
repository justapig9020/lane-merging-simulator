mod vehicle;

use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy_physimple::prelude::*;
use pid::Pid;
use vehicle::Destination;
use vehicle::Vehicle;
use vehicle::VEHICLE_SIZE;

const SCALE: f32 = 5.0;

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
        .add_system(vehicle_update_speed_sys)
        .add_system(vehicle_update_direction)
        .add_system(vehicle_movement_sys.after(vehicle_update_speed_sys))
        .add_system(move_vehicle_sys.after(vehicle_movement_sys))
        .add_system(arrive_sys.after(move_vehicle_sys))
        .add_system(set_target_sys);
    app.run();
}

fn setup_sys(mut coms: Commands, a_server: Res<AssetServer>) {
    coms.spawn_bundle(Camera2dBundle::default());

    // Vehicle
    let icon: Handle<Image> = a_server.load("vehicle.png");
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
    .insert(Vehicle::default())
    .insert(Destination::default())
    .insert(icon);
}

fn vehicle_update_speed_sys(
    time: Res<Time>,
    mut q: Query<(&Transform, &mut Vehicle, &Destination)>,
) {
    for (tran, mut veh, dest) in q.iter_mut() {
        if let Some((_, dest)) = dest.next() {
            let veh_position = tran.translation.truncate();
            println!("veh: {veh_position}");
            println!("dest: {dest}");
            let diff = *dest - veh_position;
            let forward = diff.dot(veh.direction) >= 0.0;
            let error = if forward {
                diff.length() * -1.0
            } else {
                diff.length()
            } / SCALE;
            println!("error: {error}");
            veh.update_speed(error, time.delta_seconds());
        }
    }
}

fn vehicle_update_direction(
    time: Res<Time>,
    mut q: Query<(&mut Transform, &mut Vehicle, &Destination)>,
) {
    for (mut tran, mut veh, dest) in q.iter_mut() {
        if let Some((_, dest)) = dest.next() {
            let local = tran.translation.truncate();
            let diff = (*dest - local).normalize_or_zero();
            if diff.length() != 0.0 {
                let veh_direction = veh.direction;
                let angle = veh_direction.angle_between(diff);
                let angle = if angle >= 0.0 {
                    f32::min(angle, 0.7)
                } else {
                    f32::max(angle, -0.7)
                } * time.delta_seconds();
                let quat = Quat::from_axis_angle(Vec3::Z, angle);
                tran.rotation *= quat;
                let rotate = Vec2::from_angle(angle);
                veh.direction = rotate.rotate(veh.direction);
            }
        }
    }
}

fn vehicle_movement_sys(time: Res<Time>, mut q: Query<(&mut Vel, &Vehicle)>) {
    for (mut vel, veh) in q.iter_mut() {
        let speed = veh.current_speed();
        let direction = veh.direction.normalize_or_zero();
        vel.0 = vel.0.lerp(direction * speed, time.delta_seconds() * 5.0);
        vel.0 += direction * speed * time.delta_seconds();
    }
}

fn move_vehicle_sys(time: Res<Time>, mut q: Query<(&Vel, &mut Transform)>) {
    for (v, mut t) in q.iter_mut() {
        t.translation += v.0.extend(0.0) * time.delta_seconds() * SCALE;
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
    mut commands: Commands,
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
                    let entity = commands
                        .spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                custom_size: Some(Vec2::splat(VEHICLE_SIZE / 3.0)),
                                color: Color::RED,
                                ..default()
                            },
                            transform: Transform {
                                translation: cursor.extend(0.0),
                                ..default()
                            },
                            ..Default::default()
                        })
                        .insert_bundle(SensorBundle {
                            shape: CollisionShape::Square(Square::size(Vec2::splat(
                                VEHICLE_SIZE / 3.0,
                            ))),
                            ..Default::default()
                        })
                        .id();
                    for mut dest in q_dest.iter_mut() {
                        dest.push(entity.clone(), cursor);
                    }
                }
            }
            _ => {}
        }
    }
}

fn arrive_sys(
    mut commands: Commands,
    mut dest: Query<&mut Destination>,
    q: Query<(Entity, &Sensor)>,
) {
    for (ent, s) in q.iter() {
        for &d in s.bodies.iter() {
            if let Ok(mut dest) = dest.get_mut(d) {
                if let Some((dest_ent, dest_position)) = dest.next() {
                    if *dest_ent == ent {
                        println!("Arrive {dest_position}!!");
                        dest.arrive();
                        commands.entity(ent).despawn();
                        println!("{:?}", dest);
                    }
                }
            }
        }
    }
}
