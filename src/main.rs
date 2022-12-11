mod vehicle;

use vehicle::Vehicle;

const SPEED: f64 = 60.0;
const MAX_SPEED: f64 = 80.0;
const MAX_ACCELERATION: f64 = 6.0;

fn main() {
    let veh = Vehicle::new(SPEED, MAX_SPEED, MAX_ACCELERATION);
    for i in (100..=1000).step_by(100) {
        let eat = veh.eariliest_arrival_time(i as f64);
        println!("{i}: {eat}");
    }
}
