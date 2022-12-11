mod vehicle;

use bevy::prelude::*;
use bevy_physimple::prelude::*;
use vehicle::Vehicle;

const SPEED: f64 = 60.0;
const MAX_SPEED: f64 = 80.0;
const MAX_ACCELERATION: f64 = 6.0;
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
    app.add_system(bevy::window::close_on_esc);
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
    .insert(Vehicle::new(SPEED, MAX_SPEED, MAX_ACCELERATION));
}
