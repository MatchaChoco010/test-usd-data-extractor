use usd_data_extractor::*;
use winit::{
    event::*,
    keyboard::{Key, NamedKey},
};

mod renderer;

fn show_data(data: BridgeData) {
    match data {
        BridgeData::Message(s) => println!("{}", s),
        BridgeData::TimeCodeRange(start, end) => println!("TimeCodeRange: {start} - {end}"),
        BridgeData::CreateMesh(path) => println!("{path} [CreateMesh]"),
        BridgeData::TransformMatrix(path, matrix) => {
            println!("{path} [TransformMatrix]");
            for r in 0..4 {
                print!("    ");
                for c in 0..4 {
                    print!("{:+6.4} ", matrix[r * 4 + c]);
                }
                println!();
            }
        }
        BridgeData::MeshData(path, data) => {
            println!("{path} [MeshData]");

            if data.left_handed {
                println!("    [LeftHanded]: true");
            } else {
                println!("    [LeftHanded]: false");
            }

            {
                println!(
                    "    [Points], len: {}, interpolation: {:?}",
                    data.points_data.len() / 3,
                    data.points_interpolation
                );
                print!("        ");
                for i in 0..9.min(data.points_data.len()) {
                    if i % 3 == 0 {
                        print!("(");
                    }
                    print!("{:+6.4} ", data.points_data[i]);
                    if i % 3 == 2 {
                        print!("), ");
                    }
                }
                println!("...");
            }

            if data.normals_data.is_some() {
                println!(
                    "    [Normals], len: {}, interpolation: {:?}",
                    data.normals_data.as_ref().unwrap().len() / 3,
                    data.normals_interpolation.as_ref().unwrap()
                );
                print!("        ");
                for i in 0..9.min(data.normals_data.as_ref().unwrap().len()) {
                    if i % 3 == 0 {
                        print!("(");
                    }
                    print!("{:+6.4} ", data.normals_data.as_ref().unwrap()[i]);
                    if i % 3 == 2 {
                        print!("), ");
                    }
                }
                println!("...");
            }

            if data.uvs_data.is_some() {
                println!(
                    "    [UVs], len: {}, interpolation: {:?}",
                    data.uvs_data.as_ref().unwrap().len() / 2,
                    data.uvs_interpolation.as_ref().unwrap()
                );
                print!("        ");
                for i in 0..6.min(data.uvs_data.as_ref().unwrap().len()) {
                    if i % 2 == 0 {
                        print!("(");
                    }
                    print!("{:+6.4} ", data.uvs_data.as_ref().unwrap()[i]);
                    if i % 2 == 1 {
                        print!("), ");
                    }
                }
                println!("...");
            }

            {
                println!(
                    "    [FaceVertexIndices], len: {}",
                    data.face_vertex_indices.len()
                );
                print!("        ");
                for i in 0..6.min(data.face_vertex_indices.len()) {
                    print!("{}, ", data.face_vertex_indices[i]);
                }
                println!("...");
            }

            {
                println!(
                    "    [FaceVertexCount], len: {}",
                    data.face_vertex_counts.len()
                );
                print!("        ");
                for i in 0..6.min(data.face_vertex_counts.len()) {
                    print!("{}, ", data.face_vertex_counts[i]);
                }
                println!("...");
            }
        }
        BridgeData::DestroyMesh(path) => println!("{path} [DestroyMesh]"),
    }
}

fn main() -> Result<(), winit::error::EventLoopError> {
    env_logger::init();

    let mut usd_data_extractor = UsdDataExtractor::new("./test-usd/test.usda");

    println!("Extracting USD data... (TimeCode: 1.0)");
    let diff = usd_data_extractor.extract(1.0);
    for data in diff {
        show_data(data);
    }

    println!("Extracting USD data... (TimeCode: 15.0)");
    let diff = usd_data_extractor.extract(15.0);
    for data in diff {
        show_data(data);
    }

    println!("Destroying USD data extractor...");
    let diff = usd_data_extractor.destroy();
    for data in diff {
        show_data(data);
    }

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

    let mut renderer = pollster::block_on(renderer::Renderer::new(&window));

    event_loop.run(|event, target| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
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
                renderer.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                renderer.change_scale_factor(*scale_factor as f32);
            }
            WindowEvent::RedrawRequested => match renderer.draw() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => renderer.resize(window.inner_size()),
                Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                Err(e) => eprintln!("{:?}", e),
            },
            _ => {}
        },
        _ => {}
    })
}
