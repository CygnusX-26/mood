use nalgebra::{Matrix3, Matrix4, Point3, Rotation3, Vector3};
use wgpu::{Buffer, Device, Queue, RenderPass};

use crate::{network::player_state::TimedPlayerState, renderer::Renderer};

use super::{Mesh, model_instance::RawInstance};
use wgpu::util::DeviceExt;

pub struct PlayerModel {
    pub head_mesh: Mesh,
    pub body_mesh: Mesh,
    pub head_instances: Vec<RawInstance>,
    pub body_instances: Vec<RawInstance>,
    pub head_instance_buffer: Buffer,
    pub body_instance_buffer: Buffer,
    pub head_num_instances: u32,
    pub body_num_instances: u32,
}

impl PlayerModel {
    const NO_INSTANCES: u32 = 0;
    pub fn new(
        device: &Device,
        player_states: &[TimedPlayerState],
        head_mesh: Mesh,
        body_mesh: Mesh,
    ) -> Self {
        let mut head_instances: Vec<RawInstance> = player_states
            .iter()
            .map(Self::compute_head_instance)
            .collect();
        let head_num_instances = head_instances.len() as u32;
        head_instances.resize(
            Renderer::MAX_PLAYERS as usize,
            RawInstance {
                model_mat: Matrix4::identity().into(),
                normal_mat: Matrix3::identity().into(),
            },
        );
        let mut body_instances: Vec<RawInstance> = player_states
            .iter()
            .map(Self::compute_body_instance)
            .collect();
        let body_num_instances = body_instances.len() as u32;
        body_instances.resize(
            Renderer::MAX_PLAYERS as usize,
            RawInstance {
                model_mat: Matrix4::identity().into(),
                normal_mat: Matrix3::identity().into(),
            },
        );

        let head_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Player Head Instance Buffer"),
            contents: bytemuck::cast_slice(&head_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let body_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Player Body Instance Buffer"),
            contents: bytemuck::cast_slice(&body_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            head_mesh,
            body_mesh,
            head_instances,
            body_instances,
            head_instance_buffer,
            body_instance_buffer,
            head_num_instances,
            body_num_instances,
        }
    }

    pub fn update(&mut self, queue: &Queue, player_states: &[TimedPlayerState]) {
        self.head_instances = player_states
            .iter()
            .map(Self::compute_head_instance)
            .collect();
        self.body_instances = player_states
            .iter()
            .map(Self::compute_body_instance)
            .collect();
        self.head_num_instances = self.head_instances.len() as u32;
        self.body_num_instances = self.body_instances.len() as u32;
        queue.write_buffer(
            &self.head_instance_buffer,
            0,
            bytemuck::cast_slice(&self.head_instances),
        );
        queue.write_buffer(
            &self.body_instance_buffer,
            0,
            bytemuck::cast_slice(&self.body_instances),
        );
    }

    pub fn draw(&self, render_pass: &mut RenderPass) {
        if self.head_num_instances == Self::NO_INSTANCES
            || self.body_num_instances == Self::NO_INSTANCES
        {
            return;
        }
        let head_mesh = &self.head_mesh;
        let body_mesh = &self.body_mesh;

        // draw head
        render_pass.set_vertex_buffer(1, self.head_instance_buffer.slice(..));
        render_pass.set_vertex_buffer(0, head_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(head_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..head_mesh.num_elements, 0, 0..self.head_num_instances);

        // draw body
        render_pass.set_vertex_buffer(1, self.body_instance_buffer.slice(..));
        render_pass.set_vertex_buffer(0, body_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(body_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..body_mesh.num_elements, 0, 0..self.body_num_instances);
    }

    fn compute_head_instance(timed_player_state: &TimedPlayerState) -> RawInstance {
        let dt = timed_player_state.time.elapsed().as_secs_f32();
        let player_state = timed_player_state.player_state;
        let yaw_rotation = Matrix4::from(Rotation3::from_axis_angle(
            &Vector3::y_axis(),
            player_state.yaw,
        ));
        let pitch_rotation = Matrix4::from(Rotation3::from_axis_angle(
            &Vector3::x_axis(),
            -player_state.pitch,
        ));
        let new_pos =
            Point3::from(player_state.position) + Vector3::from(player_state.velocity) * dt;
        let rotation_mat = yaw_rotation * pitch_rotation;

        let model_mat = Matrix4::new_translation(&new_pos.coords) * rotation_mat;
        let normal_mat = rotation_mat.fixed_resize::<3, 3>(0.0);
        RawInstance {
            model_mat: model_mat.into(),
            normal_mat: normal_mat.into(),
        }
    }

    fn compute_body_instance(timed_player_state: &TimedPlayerState) -> RawInstance {
        let dt = timed_player_state.time.elapsed().as_secs_f32();
        let player_state = timed_player_state.player_state;
        let new_pos =
            Point3::from(player_state.position) + Vector3::from(player_state.velocity) * dt;
        let yaw_rotation = Matrix4::from(Rotation3::from_axis_angle(
            &Vector3::y_axis(),
            player_state.body_yaw,
        ));

        let model_mat = Matrix4::new_translation(&new_pos.coords) * yaw_rotation;
        let normal_mat = yaw_rotation.fixed_resize::<3, 3>(0.0);
        RawInstance {
            model_mat: model_mat.into(),
            normal_mat: normal_mat.into(),
        }
    }
}
