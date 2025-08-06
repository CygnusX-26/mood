use nalgebra::{Point3, Vector3};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};
use std::time::Duration;

use crate::camera::Camera;

use super::{
    bounding_box::BoundingBox, collision_manager::CollisionManager,
    player_controller::PlayerController,
};

pub struct Player {
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
    sensitivity: f32,
    speed: f32,
    jump_strength: f32,
    hitbox: BoundingBox,
    is_on_ground: bool,
    pub camera: Camera,
    pub yaw: f32,
    pub pitch: f32,
    pub body_yaw: f32,
}

impl Player {
    const GRAVITY: f32 = 0.1;
    const SLOW_DOWN: f32 = 0.0;
    const MAX_HEAD_TURN: f32 = FRAC_PI_4;
    const BODY_ROTATION_SPEED: f32 = 5.0;

    pub fn new(
        sensitivity: f32,
        speed: f32,
        jump_strength: f32,
        hitbox_width: f32,
        hitbox_height: f32,
        camera: Camera,
    ) -> Self {
        let position = camera.position;
        Self {
            position,
            velocity: Vector3::zeros(),
            sensitivity,
            speed,
            jump_strength,
            hitbox: BoundingBox {
                top_left: Point3::new(
                    position.x - (hitbox_width / 2.0),
                    position.y,
                    position.z - (hitbox_width / 2.0),
                ),
                bottom_right: Point3::new(
                    position.x + (hitbox_width / 2.0),
                    position.y - hitbox_height,
                    position.z + (hitbox_width / 2.0),
                ),
                collide_on_top: false,
            },
            is_on_ground: false,
            camera,
            pitch: 0.0,
            yaw: 0.0,
            body_yaw: 0.0,
        }
    }

    pub fn update(
        &mut self,
        dt: Duration,
        collision_manager: &mut CollisionManager,
        player_controller: &mut PlayerController,
    ) {
        let sens = self.sensitivity * dt.as_secs_f32();
        self.velocity.x *= Self::SLOW_DOWN;
        self.velocity.z *= Self::SLOW_DOWN;
        let yaw_diff = self.yaw - self.body_yaw;
        let normalized_yaw = Self::normalize_yaw(yaw_diff);
        if let Some(delta_mouse_pos) = player_controller.delta_mouse_pos {
            self.yaw -= delta_mouse_pos.0 * sens;
            self.pitch -= delta_mouse_pos.1 * sens;
            player_controller.delta_mouse_pos = None;
            let max_pitch = FRAC_PI_2 - 0.01;
            self.pitch = self.pitch.clamp(-max_pitch, max_pitch);
            self.camera.rotate_camera(self.pitch, self.yaw);

            self.body_yaw += Self::body_yaw_change(normalized_yaw, dt);
        }

        let camera_position = self.camera.position;
        let camera_target = self.camera.target;
        let camera_up = self.camera.up;

        let looking_at = (camera_target - camera_position).normalize();
        let left = camera_up.cross(&looking_at).normalize();
        let forward = left.cross(&camera_up).normalize();
        let mut delta_velocity = Vector3::zeros();
        if player_controller.is_w_pressed {
            self.body_yaw = self.yaw;
            delta_velocity += forward;
        }
        if player_controller.is_s_pressed {
            self.body_yaw = self.yaw;
            delta_velocity -= forward;
        }
        if player_controller.is_a_pressed {
            self.body_yaw = self.yaw;
            delta_velocity += left;
        }
        if player_controller.is_d_pressed {
            self.body_yaw = self.yaw;
            delta_velocity -= left;
        }
        if player_controller.is_space_pressed && self.is_on_ground {
            self.velocity.y += self.jump_strength;
        }
        self.velocity.y -= Self::GRAVITY;
        let mut movement_velocity = Vector3::zeros();
        if let Some(normalized_delta_velocity) = delta_velocity.try_normalize(0.0) {
            movement_velocity = normalized_delta_velocity * self.speed;
        }
        self.velocity.x += movement_velocity.x;
        self.velocity.z += movement_velocity.z;
        let intended_displacement = self.velocity * dt.as_secs_f32();
        let actual_displacement =
            collision_manager.move_player(&mut self.hitbox, intended_displacement);
        if actual_displacement.y == 0.0 && intended_displacement.y < 0.0 {
            self.velocity.y = 0.0;
            self.is_on_ground = true;
        } else {
            self.is_on_ground = false;
        }
        self.velocity = actual_displacement / dt.as_secs_f32();
        self.camera.move_camera(actual_displacement);
        self.position += actual_displacement;
    }

    fn normalize_yaw(mut yaw: f32) -> f32 {
        while yaw <= -PI {
            yaw += 2.0 * PI;
        }
        while yaw > PI {
            yaw -= 2.0 * PI;
        }
        yaw
    }

    fn body_yaw_change(goal_yaw: f32, dt: Duration) -> f32 {
        if goal_yaw.abs() > Self::MAX_HEAD_TURN {
            let catch_up_amount = Self::BODY_ROTATION_SPEED * dt.as_secs_f32();

            if goal_yaw > Self::MAX_HEAD_TURN {
                catch_up_amount.min(goal_yaw - Self::MAX_HEAD_TURN)
            } else {
                -catch_up_amount.min(-goal_yaw - Self::MAX_HEAD_TURN)
            }
        } else {
            0.0
        }
    }
}
