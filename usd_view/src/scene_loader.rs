use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};
use usd_data_extractor::*;

use crate::render_scene::RenderScene;

#[derive(Debug)]
pub struct TimeCodeRange {
    pub start: i64,
    pub end: i64,
}

#[derive(Debug)]
struct RenderSettings {
    settings_paths: Vec<String>,
    product_paths: Vec<String>,
    active_settings_path: Option<String>,
    active_product_path: Option<String>,
}

#[derive(Debug)]
struct SyncItems {
    scene: RenderScene,
    render_settings: RenderSettings,
    time_code_range: Option<TimeCodeRange>,
}

enum UsdSceneExtractorMessage {
    LoadUsd(String),
    SetTimeCode(i64),
    SetActiveRenderSettings(Option<String>),
    SetActiveRenderProduct(Option<String>),
    Stop,
}

struct UsdSceneExtractorTask {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    usd_data_extractor: Option<UsdSceneExtractor>,

    sync_items: Arc<Mutex<SyncItems>>,

    active_render_settings_path: Option<String>,
    active_render_product_path: Option<String>,
}
impl UsdSceneExtractorTask {
    // 裏でusd読み込みのために走っているスレッドのエントリーポイント。
    // スレッドでは無限ループ内で外部からのメッセージを監視し、、
    // それぞれのメッセージが来たらload_usd, set_time_code, stopを呼び出す。
    pub fn run(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        sync_items: Arc<Mutex<SyncItems>>,
        receiver: Receiver<UsdSceneExtractorMessage>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let mut task = Self {
                device,
                queue,
                usd_data_extractor: None,
                sync_items,
                active_render_settings_path: None,
                active_render_product_path: None,
            };

            loop {
                let mut filename = None;
                let mut time_code = None;
                let mut active_render_settings_path = None;
                let mut active_render_product_path = None;
                while let Ok(message) = receiver.recv() {
                    match message {
                        UsdSceneExtractorMessage::LoadUsd(file) => {
                            filename = Some(file);
                            break;
                        }
                        UsdSceneExtractorMessage::SetTimeCode(tc) => {
                            time_code = Some(tc);
                            break;
                        }
                        UsdSceneExtractorMessage::SetActiveRenderSettings(path) => {
                            active_render_settings_path = Some(path);
                            break;
                        }
                        UsdSceneExtractorMessage::SetActiveRenderProduct(path) => {
                            active_render_product_path = Some(path);
                            break;
                        }
                        UsdSceneExtractorMessage::Stop => {
                            return;
                        }
                    }
                }

                if let Some(filename) = filename {
                    task.load_usd(&filename);
                }

                if let Some(time_code) = time_code {
                    task.set_time_code(time_code);
                }

                if let Some(path) = active_render_settings_path {
                    task.set_active_render_settings_path(path);
                }

                if let Some(path) = active_render_product_path {
                    task.set_active_render_product_path(path);
                }
            }
        })
    }

    // 裏でusd読み込みのために走っているスレッドで、USDファイルのロードボタンが押されていたら呼び出されるメソッド。
    // 新しくUsdDataExtractorを作成し、syncしているシーン情報などを初期化する。
    fn load_usd(&mut self, filename: &str) {
        let mut sync_items = self.sync_items.lock().unwrap();
        self.usd_data_extractor = UsdSceneExtractor::new(filename)
            .inspect_err(|_| eprintln!("Failed to open USD file: {filename}"))
            .ok();
        let (start, end) = self
            .usd_data_extractor
            .as_ref()
            .map(|e| e.time_code_range())
            .unwrap_or((0.0, 0.0));
        sync_items.scene = RenderScene::new(Arc::clone(&self.device), Arc::clone(&self.queue));
        sync_items.render_settings = RenderSettings {
            settings_paths: Vec::new(),
            product_paths: Vec::new(),
            active_settings_path: None,
            active_product_path: None,
        };
        sync_items.time_code_range = Some(TimeCodeRange {
            start: start as i64,
            end: end as i64,
        });
    }

    // 裏でusd読み込みのために走っているスレッドでtime_codeが変更された際に呼び出されるメソッド。
    // UsdDataExtractorからtime_codeに対応するデータを取得し、
    // UsdSceneExtractorのメンバ変数に反映する。
    fn set_time_code(&mut self, time_code: i64) {
        let Some(usd_data_extractor) = &mut self.usd_data_extractor else {
            return;
        };

        let mut scene = self.sync_items.lock().unwrap();
        let scene = &mut scene.scene;

        let diff = usd_data_extractor.extract(time_code as f64);
        for item in diff.items {
            match item {
                SceneDiffItem::MeshCreated(path, transform_matrix, mesh_data) => {
                    scene.add_mesh(path.into(), transform_matrix, mesh_data);
                }
                SceneDiffItem::MeshDestroyed(path) => {
                    scene.remove_mesh(path.into());
                }
                SceneDiffItem::MeshTransformMatrixDirtied(path, transform_matrix) => {
                    scene.update_mesh_transform_matrix(path.into(), transform_matrix);
                }
                SceneDiffItem::MeshDataDirtied(path, mesh_data) => {
                    scene.update_mesh_data(path.into(), mesh_data);
                }
                SceneDiffItem::SphereLightAddOrUpdate(path, light) => {
                    scene.insert_sphere_light(path.into(), light);
                }
                SceneDiffItem::SphereLightDestroyed(path) => {
                    scene.remove_sphere_light(path.into());
                }
                SceneDiffItem::DistantLightAddOrUpdate(path, light) => {
                    scene.insert_distant_light(path.into(), light);
                }
                SceneDiffItem::DistantLightDestroyed(path) => {
                    scene.remove_distant_light(path.into());
                }
            }
        }
    }

    // 裏でusd読み込みのために走っているスレッドでSetActiveRenderSettingsが呼ばれた際に呼び出されるメソッド。
    // 渡されたpathがステージに存在しているかを確認している。
    // 存在する場合はRenderSettingsをactiveに設定する。
    // 存在しない場合はactiveなRenderSettingsとRenderProductの設定をクリアする。
    // どちらの場合も現在のカメラのパスをシーンからクリアする。
    fn set_active_render_settings_path(&mut self, path: Option<String>) {
        // match path {
        //     Some(path) => {
        //         let has_path = self.render_settings.contains_key(&path);
        //         if has_path {
        //             self.active_render_settings_path = Some(path.clone());
        //             self.active_render_product_path = None;
        //         } else {
        //             self.active_render_settings_path = None;
        //             self.active_render_product_path = None;
        //         }
        //     }
        //     None => {
        //         self.active_render_settings_path = None;
        //         self.active_render_product_path = None;
        //     }
        // }
    }

    // 裏でusd読み込みのために走っているスレッドでSetActiveRenderProductが呼ばれた際に呼び出されるメソッド。
    // 渡されたpathが現在のアクティブなRenderSettingsに存在しているかを確認している。
    // 存在する場合はactiveなRenderProductの設定を更新し、そのRenderProductにあるカメラのパスをシーンに設定する。
    // 存在しない場合はactiveなRenderProductの設定とシーンのカメラのパスをクリアする。

    fn set_active_render_product_path(&mut self, path: Option<String>) {
        // match path {
        //     Some(path) => {
        //         let Some(render_settings) = self.render_settings.get(&path) else {
        //             self.active_render_product_path = None;
        //             return;
        //         };
        //         let has_path = render_settings.product_paths.contains(&path);
        //         if has_path {
        //             self.active_render_product_path = Some(path.clone());
        //         } else {
        //             self.active_render_product_path = None;
        //         }
        //     }
        //     None => {
        //         self.active_render_product_path = None;
        //     }
        // }
    }

    // // アクティブなRenderSettingsのRenderProductからカメラのパスを取得して設定する。
    // fn sync_camera(&mut self, scene: &mut Scene) {
    //     // アクティブなRenderProductのパスを取得する
    //     let camera_path = match &self.active_render_product_path {
    //         Some(active_render_product_path) => {
    //             match self.render_products.get(active_render_product_path) {
    //                 Some(render_product) => Some(&render_product.camera_path),
    //                 None => None,
    //             }
    //         }
    //         None => None,
    //     };

    //     // アクティブなカメラの情報を取得する
    //     let camera = match camera_path {
    //         Some(path) => self.cameras.get(path),
    //         None => None,
    //     };

    //     // カメラの情報をシーンに反映する
    //     match camera {
    //         Some(camera) => {
    //             let transform = glam::Mat4::from_cols_array(&camera.transform_matrix);
    //             let position = transform.transform_point3(Vec3::ZERO);
    //             let direction = transform.transform_vector3(Vec3::NEG_Z).normalize();
    //             scene.camera.view_matrix =
    //                 glam::Mat4::look_at_rh(position, position + direction, Vec3::Y);
    //             let fovy = 2.0 * (camera.vertical_aperture / 2.0 / camera.focal_length).atan();
    //             scene.camera.fovy = fovy;
    //         }
    //         None => {
    //             scene.camera.view_matrix = glam::Mat4::look_at_rh(
    //                 Vec3::new(0.0, 1.8, 5.0),
    //                 Vec3::new(0.0, 0.8, 0.0),
    //                 Vec3::Y,
    //             );
    //             scene.camera.fovy = 60.0_f32.to_radians();
    //         }
    //     }
    // }
}

pub struct SceneLoader {
    sync_item: Arc<Mutex<SyncItems>>,
    message_sender: Sender<UsdSceneExtractorMessage>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}
impl SceneLoader {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let scene = RenderScene::new(Arc::clone(&device), Arc::clone(&queue));
        let render_settings = RenderSettings {
            settings_paths: Vec::new(),
            product_paths: Vec::new(),
            active_settings_path: None,
            active_product_path: None,
        };
        let sync_item = Arc::new(Mutex::new(SyncItems {
            scene,
            render_settings,
            time_code_range: None,
        }));
        let (message_sender, message_receiver) = channel();

        let join_handle =
            UsdSceneExtractorTask::run(device, queue, Arc::clone(&sync_item), message_receiver);

        Self {
            sync_item,
            message_sender,
            join_handle: Some(join_handle),
        }
    }

    pub fn load_usd(&self, filename: &str) {
        self.message_sender
            .send(UsdSceneExtractorMessage::LoadUsd(filename.to_string()))
            .unwrap();
    }

    pub fn set_time_code(&self, time_code: i64) {
        self.message_sender
            .send(UsdSceneExtractorMessage::SetTimeCode(time_code))
            .unwrap();
    }

    pub fn get_time_code_range(&self) -> (i64, i64) {
        let sync_item = self.sync_item.lock().unwrap();
        sync_item
            .time_code_range
            .as_ref()
            .map(|range| (range.start as i64, range.end as i64))
            .unwrap_or((0, 0))
    }

    pub fn read_scene(&self, f: impl FnOnce(&RenderScene)) {
        let sync_item = self.sync_item.lock().unwrap();
        f(&sync_item.scene);
    }

    pub fn get_render_settings_paths(&self) -> Vec<String> {
        let sync_item = self.sync_item.lock().unwrap();
        sync_item
            .render_settings
            .settings_paths
            .iter()
            .cloned()
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn set_active_render_settings_path(&self, path: Option<&str>) {
        self.message_sender
            .send(UsdSceneExtractorMessage::SetActiveRenderSettings(
                path.map(|s| s.to_string()),
            ))
            .unwrap();
    }

    pub fn get_active_render_settings_path(&self) -> Option<String> {
        let sync_item = self.sync_item.lock().unwrap();
        sync_item.render_settings.active_settings_path.clone()
    }

    pub fn get_render_product_paths(&self) -> Vec<String> {
        let sync_item = self.sync_item.lock().unwrap();
        sync_item.render_settings.product_paths.clone()
    }

    pub fn set_active_render_product_path(&self, path: Option<&str>) {
        self.message_sender
            .send(UsdSceneExtractorMessage::SetActiveRenderProduct(
                path.map(|s| s.to_string()),
            ))
            .unwrap();
    }

    pub fn get_active_render_product_path(&self) -> Option<String> {
        let sync_item = self.sync_item.lock().unwrap();
        sync_item.render_settings.active_product_path.clone()
    }
}
impl Drop for SceneLoader {
    fn drop(&mut self) {
        self.message_sender
            .send(UsdSceneExtractorMessage::Stop)
            .unwrap();
        let join_handle = self.join_handle.take().unwrap();
        join_handle.join().unwrap();
    }
}
