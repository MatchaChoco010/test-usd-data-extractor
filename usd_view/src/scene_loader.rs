use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};
use usd_data_extractor::*;

pub struct TimeCodeRange {
    pub start: i32,
    pub end: i32,
}

pub struct Scene {
    pub range: Option<TimeCodeRange>,
}

pub struct SceneLoader {
    scene: Arc<Mutex<Scene>>,
    open_usd_sender: Sender<String>,
    time_code_sender: Sender<i32>,
    stop_sender: Sender<()>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}
impl SceneLoader {
    pub fn new() -> Self {
        let scene = Arc::new(Mutex::new(Scene { range: None }));
        let (time_code_sender, time_code_receiver) = channel();
        let (open_usd_sender, open_usd_receiver) = channel();
        let (stop_sender, stop_receiver) = channel();

        let join_handle = UsdSceneExtractorTask::run(
            Arc::clone(&scene),
            open_usd_receiver,
            time_code_receiver,
            stop_receiver,
        );

        Self {
            scene,
            open_usd_sender,
            time_code_sender,
            stop_sender,
            join_handle: Some(join_handle),
        }
    }

    pub fn load_usd(&self, filename: &str) {
        self.open_usd_sender.send(filename.to_string()).unwrap();
    }

    pub fn set_time_code(&self, time_code: i32) {
        self.time_code_sender.send(time_code).unwrap();
    }

    pub fn read_scene(&self, f: impl FnOnce(&Scene)) {
        let scene = self.scene.lock().unwrap();
        f(&scene);
    }
}
impl Drop for SceneLoader {
    fn drop(&mut self) {
        self.stop_sender.send(()).unwrap();
        let join_handle = self.join_handle.take().unwrap();
        join_handle.join().unwrap();
    }
}

pub struct UsdSceneExtractorTask {
    usd_data_extractor: Option<UsdDataExtractor>,
    scene: Arc<Mutex<Scene>>,
}
impl UsdSceneExtractorTask {
    pub fn run(
        scene: Arc<Mutex<Scene>>,
        open_usd_receiver: Receiver<String>,
        time_code_receiver: Receiver<i32>,
        stop_receiver: Receiver<()>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let mut task = Self {
                usd_data_extractor: None,
                scene,
            };

            loop {
                let mut filename = None;
                while let Ok(file) = open_usd_receiver.try_recv() {
                    filename = Some(file);
                }
                if let Some(filename) = filename {
                    task.load_usd(&filename);
                }

                let mut time_code = None;
                while let Ok(tc) = time_code_receiver.try_recv() {
                    time_code = Some(tc);
                }
                if let Some(time_code) = time_code {
                    task.set_time_code(time_code);
                }

                if stop_receiver.try_recv().is_ok() {
                    break;
                }
            }
        })
    }

    fn load_usd(&mut self, filename: &str) {
        let mut scene = self.scene.lock().unwrap();
        self.usd_data_extractor = Some(UsdDataExtractor::new(filename));
        *scene = Scene { range: None };
    }

    fn set_time_code(&mut self, time_code: i32) {
        let Some(usd_data_extractor) = &mut self.usd_data_extractor else {
            return;
        };

        let diff = usd_data_extractor.extract(time_code as f64);
        for data in &diff {
            show_data(data);
        }
        for data in diff {
            match data {
                BridgeData::TimeCodeRange(start, end) => {
                    let mut scene = self.scene.lock().unwrap();
                    scene.range = Some(TimeCodeRange {
                        start: start as i32,
                        end: end as i32,
                    });
                }
                _ => (),
            }
        }
    }
}

fn show_data(data: &BridgeData) {
    match &data {
        &BridgeData::Message(s) => println!("{}", s),
        &BridgeData::TimeCodeRange(start, end) => println!("TimeCodeRange: {start} - {end}"),
        &BridgeData::CreateMesh(path) => println!("{path} [CreateMesh]"),
        &BridgeData::TransformMatrix(path, matrix) => {
            println!("{path} [TransformMatrix]");
            for r in 0..4 {
                print!("    ");
                for c in 0..4 {
                    print!("{:+6.4} ", matrix[r * 4 + c]);
                }
                println!();
            }
        }
        &BridgeData::MeshData(path, data) => {
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
        &BridgeData::DestroyMesh(path) => println!("{path} [DestroyMesh]"),
    }
}
