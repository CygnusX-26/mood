use log::error;
use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
    window::WindowAttributes,
};

use crate::renderer::Renderer;

#[derive(Default)]
pub struct AppState {
    renderer: Option<Renderer>,
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("Mood"))
                .unwrap(),
        );

        self.renderer = match pollster::block_on(Renderer::new(window.clone())) {
            Ok(r) => Some(r),
            Err(e) => {
                error!("{e}");
                std::process::exit(1);
            }
        };
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                renderer.update();
                match renderer.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = renderer.get_window().inner_size();
                        renderer.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {e}");
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
                if renderer
                    .get_mut_camera()
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
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        match event {
            DeviceEvent::MouseMotion { delta } => {
                renderer.get_mut_camera().handle_mouse(delta);
            }
            _ => {}
        }
    }
}
