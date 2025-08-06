use log::error;
use std::{net::Ipv4Addr, sync::Arc, time::Instant};

use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowAttributes,
};

use crate::{network::Network, renderer::Renderer};

#[derive(Default)]
pub struct AppState {
    renderer: Option<Renderer>,
    prev_frame_time: Option<Instant>,
    network_handler: Option<Network>,
}

impl AppState {
    fn cleanup(&self, event_loop: &ActiveEventLoop) {
        if let Some(ref network_handler) = self.network_handler {
            network_handler.send_player_leave().unwrap();
        };
        event_loop.exit();
    }
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("Mood"))
                .unwrap(),
        );

        self.renderer = match pollster::block_on(Renderer::new(
            window.clone(),
            String::from("client/src/model/maps/map_1.json"),
        )) {
            Ok(r) => Some(r),
            Err(e) => {
                error!("{e}");
                std::process::exit(1);
            }
        };
        self.network_handler = match Network::new(Ipv4Addr::new(127, 0, 0, 1), 8003) {
            Ok(nh) => Some(nh),
            Err(_) => {
                error!("A network setup error occurred!");
                None
            }
        };
        if let Some(ref network_handler) = self.network_handler {
            network_handler.send_player_join().unwrap();
        }
        self.prev_frame_time = Some(Instant::now());
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        //send movement data
        if let Some(ref mut network_handler) = self.network_handler {
            network_handler.poll();
            let player = renderer.get_player();
            if network_handler
                .send_player_move(
                    player.position.into(),
                    player.velocity.into(),
                    player.pitch,
                    player.yaw,
                    player.body_yaw,
                )
                .is_err()
            {
                error!("Server-Client desync!");
            };
        }
        match event {
            WindowEvent::CloseRequested => {
                self.cleanup(event_loop);
            }
            WindowEvent::RedrawRequested => {
                renderer.update(
                    self.prev_frame_time.unwrap_or_else(Instant::now).elapsed(),
                    &self.network_handler,
                );
                self.prev_frame_time = Some(Instant::now());
                match renderer.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = renderer.get_window().inner_size();
                        renderer.resize(size.width, size.height);
                    }
                    Err(e) => {
                        error!("Unable to render {e}");
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                // Special key for reloading the renderer
                if code == KeyCode::KeyB && state.is_pressed() {
                    renderer.rerender();
                    renderer.get_window().as_ref().request_redraw();
                } else if code == KeyCode::Escape && state.is_pressed() {
                    self.cleanup(event_loop);
                } else if renderer
                    .get_mut_player_controller()
                    .handle_key_held(code, state, event_loop)
                {
                    renderer.get_window().as_ref().request_redraw();
                }
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        match event {
            DeviceEvent::MouseMotion { delta } => {
                renderer.get_mut_player_controller().handle_mouse(delta);
            }
            _ => {}
        }
    }
}
