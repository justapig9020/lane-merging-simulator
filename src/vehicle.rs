use bevy::prelude::*;

#[derive(Component)]
pub struct Vehicle {
    speed: f64,
    max_speed: f64,
    max_acceleration: f64,
}

impl Vehicle {
    pub fn new(speed: f64, max_speed: f64, max_acceleration: f64) -> Self {
        Self {
            speed,
            max_speed,
            max_acceleration,
        }
    }
    pub fn eariliest_arrival_time(&self, distance: f64) -> f64 {
        let cs = self.speed;
        let ms = self.max_speed;
        let ma = self.max_acceleration;
        let t1 = (ms - cs) / ma;
        let acc_distance = cs * t1 + ma / 2.0 * t1.powi(2);
        let eat = if distance <= acc_distance {
            (-1.0 * cs + (cs.powi(2) + 2.0 * ma * distance).sqrt()) / ma
        } else {
            (distance - acc_distance + ms * t1) / ms
        };
        eat
    }
}
