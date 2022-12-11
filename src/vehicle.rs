use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_physimple::prelude::*;
use pid::Pid;

// unit:
//  distance: meter
//  time: second
const SPEED: f32 = 60.0 * 1000.0 / 3600.0;
const MAX_SPEED: f32 = 80.0 * 1000.0 / 3600.0;
const MAX_ACCELERATION: f32 = 6.0;
const MAX_BREAK: f32 = 7.35;
pub const VEHICLE_SIZE: f32 = 25.0;

/// Speed unit: km / hr
#[derive(Component, Debug)]
pub struct Vehicle {
    speed: f32,
    max_speed: f32,
    max_acceleration: f32,
    max_break: f32,
    pub direction: Vec2,
    controller: Pid<f32>,
}

impl Default for Vehicle {
    fn default() -> Self {
        let kp = 0.05;
        let ki = 0.0;
        let kd = 6.0;
        let p_limit = 100.0;
        let i_limit = 0.0;
        let d_limit = 100.0;

        let controller = Pid::new(kp, ki, kd, p_limit, i_limit, d_limit, 1.0, 0.0);
        Self {
            speed: 0.0,
            max_speed: MAX_SPEED,
            max_acceleration: MAX_ACCELERATION,
            max_break: MAX_BREAK,
            direction: Vec2 { x: 1.0, y: 0.0 },
            controller,
        }
    }
}

impl Vehicle {
    pub fn eariliest_arrival_time(&self, distance: f32) -> f32 {
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
    pub fn current_speed(&self) -> f32 {
        self.speed
    }
    pub fn update_speed(&mut self, distance: f32, delta_t: f32) {
        let result = self.controller.next_control_output(distance);
        let output = result.output;

        let acc = if output >= 0.0 {
            output * self.max_acceleration
        } else {
            output * self.max_break
        };
        self.speed = f32::max(
            f32::min(self.max_speed, self.speed + acc * delta_t),
            self.max_speed * -1.0,
        );
    }
    pub fn stop(&mut self) {
        self.speed = f32::max(0.0, self.speed - self.max_break / 2.0);
    }
}

#[derive(Component, Default, Debug)]
pub struct Destination {
    dests: VecDeque<(Entity, Vec2)>,
}

impl Destination {
    pub fn is_last(&self) -> bool {
        self.dests.len() == 0
    }
    pub fn next(&self) -> Option<&(Entity, Vec2)> {
        self.dests.get(0)
    }
    pub fn push(&mut self, entity: Entity, position: Vec2) {
        self.dests.push_back((entity, position));
    }
    pub fn arrive(&mut self) {
        self.dests.pop_front();
    }
}
