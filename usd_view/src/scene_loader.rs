use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};
use usd_data_extractor::*;

use crate::render_scene::RenderScene;

#[derive(Debug)]
pub struct TimeCodeRange {
    pub start: i64,
    pub end: i64,
}

#[derive(Debug)]
struct UsdRenderSettings {
    settings: HashMap<String, RenderSettings>,
    active_settings_path: Option<String>,
    active_product_path: Option<String>,
}

#[derive(Debug)]
struct SyncItems {
    scene: RenderScene,
    render_settings: UsdRenderSettings,
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
        sync_items.render_settings = UsdRenderSettings {
            settings: HashMap::new(),
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

        let mut sync_items = self.sync_items.lock().unwrap();
        let diff = usd_data_extractor.extract(time_code as f64);
        for item in diff.items {
            match item {
                SceneDiffItem::MeshCreated(path, transform_matrix, mesh_data) => {
                    sync_items
                        .scene
                        .add_mesh(path.into(), transform_matrix, mesh_data);
                }
                SceneDiffItem::MeshDestroyed(path) => {
                    sync_items.scene.remove_mesh(path.into());
                }
                SceneDiffItem::MeshTransformMatrixDirtied(path, transform_matrix) => {
                    sync_items
                        .scene
                        .update_mesh_transform_matrix(path.into(), transform_matrix);
                }
                SceneDiffItem::MeshDataDirtied(path, mesh_data) => {
                    sync_items.scene.update_mesh_data(path.into(), mesh_data);
                }
                SceneDiffItem::SphereLightAddOrUpdate(path, light) => {
                    sync_items.scene.insert_sphere_light(path.into(), light);
                }
                SceneDiffItem::SphereLightDestroyed(path) => {
                    sync_items.scene.remove_sphere_light(path.into());
                }
                SceneDiffItem::DistantLightAddOrUpdate(path, light) => {
                    sync_items.scene.insert_distant_light(path.into(), light);
                }
                SceneDiffItem::DistantLightDestroyed(path) => {
                    sync_items.scene.remove_distant_light(path.into());
                }
                SceneDiffItem::CameraAddOrUpdate(path, camera) => {
                    sync_items.scene.insert_camera(path.into(), camera);
                }
                SceneDiffItem::CameraDestroyed(path) => {
                    sync_items.scene.remove_camera(path.into());
                }
                SceneDiffItem::RenderSettingsAddOrUpdate(path, settings) => {
                    sync_items
                        .render_settings
                        .settings
                        .insert(path.into(), settings);
                }
                SceneDiffItem::RenderSettingsDestroyed(path) => {
                    let path: String = path.into();
                    sync_items.render_settings.settings.remove(&path);
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
        let mut sync_items = self.sync_items.lock().unwrap();
        let scene = &mut sync_items.scene;
        scene.set_active_camera_path(None);
        let render_settings = &mut sync_items.render_settings;
        match path {
            Some(path) => {
                let has_path = render_settings.settings.contains_key(&path);
                if has_path {
                    render_settings.active_settings_path = Some(path.clone());
                    render_settings.active_product_path = None;
                } else {
                    render_settings.active_settings_path = None;
                    render_settings.active_product_path = None;
                }
            }
            None => {
                render_settings.active_settings_path = None;
                render_settings.active_product_path = None;
            }
        }
    }

    // 裏でusd読み込みのために走っているスレッドでSetActiveRenderProductが呼ばれた際に呼び出されるメソッド。
    // 渡されたpathが現在のアクティブなRenderSettingsに存在しているかを確認している。
    // 存在する場合はactiveなRenderProductの設定を更新し、そのRenderProductにあるカメラのパスをシーンに設定する。
    // 存在しない場合はactiveなRenderProductの設定とシーンのカメラのパスをクリアする。
    fn set_active_render_product_path(&mut self, path: Option<String>) {
        let mut sync_items = self.sync_items.lock().unwrap();
        match path {
            Some(path) => {
                if let Some(active_settings_path) =
                    sync_items.render_settings.active_settings_path.clone()
                {
                    if !sync_items
                        .render_settings
                        .settings
                        .contains_key(&active_settings_path)
                    {
                        sync_items.render_settings.active_product_path = None;
                        sync_items.scene.set_active_camera_path(None);
                        return;
                    };
                    let render_settings = &mut sync_items.render_settings;
                    if let Some(render_product) = render_settings
                        .settings
                        .get(&active_settings_path)
                        .unwrap()
                        .render_products
                        .get(&path)
                    {
                        render_settings.active_product_path = Some(path.clone());
                        let camera_path = render_product.camera_path.clone();
                        sync_items.scene.set_active_camera_path(Some(camera_path));
                    } else {
                        sync_items.render_settings.active_product_path = None;
                        sync_items.scene.set_active_camera_path(None);
                    }
                }
            }
            None => {
                sync_items.render_settings.active_product_path = None;
                sync_items.scene.set_active_camera_path(None);
            }
        }
    }
}

pub struct SceneLoader {
    sync_item: Arc<Mutex<SyncItems>>,
    message_sender: Sender<UsdSceneExtractorMessage>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}
impl SceneLoader {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let scene = RenderScene::new(Arc::clone(&device), Arc::clone(&queue));
        let render_settings = UsdRenderSettings {
            settings: HashMap::new(),
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
        let mut ret = sync_item
            .render_settings
            .settings
            .keys()
            .map(|s| s.clone())
            .collect::<Vec<String>>();
        ret.sort();
        ret
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
        if let Some(active_settings_path) = self.get_active_render_settings_path() {
            let sync_item = self.sync_item.lock().unwrap();
            let render_settings = &sync_item.render_settings;
            if let Some(settings) = render_settings.settings.get(&active_settings_path) {
                let mut ret = settings
                    .render_products
                    .keys()
                    .map(|s| s.clone())
                    .collect::<Vec<String>>();
                ret.sort();
                return ret;
            }
        }
        vec![]
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
