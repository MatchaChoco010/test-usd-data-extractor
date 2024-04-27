use winit::{
    event::*,
    event_loop::ControlFlow,
    keyboard::{Key, NamedKey},
};

mod egui_renderer;
mod renderer;
mod scene_loader;
mod state;

fn main() -> Result<(), winit::error::EventLoopError> {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = winit::window::WindowBuilder::new()
        .with_resizable(true)
        .with_title("my usd view")
        .with_inner_size(winit::dpi::PhysicalSize {
            width: 800,
            height: 600,
        })
        .build(&event_loop)
        .unwrap();

    let mut state = pollster::block_on(state::State::new(&window));

    event_loop.run(|event, target| {
        target.set_control_flow(ControlFlow::Poll);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                state.handle_event(&window, event);
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    } => target.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        state.change_scale_factor(*scale_factor as f32);
                    }
                    WindowEvent::RedrawRequested => match state.draw(&window) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(window.inner_size()),
                        Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    },
                    _ => {}
                }
            }
            Event::AboutToWait { .. } => {
                window.request_redraw();
            }
            _ => {}
        }
    })
}
