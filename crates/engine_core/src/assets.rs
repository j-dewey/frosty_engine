//
// functions for useful asset management
//
use std::collections::VecDeque;

use frosty_alloc::FrostyAllocatable;
use hashbrown::{HashMap, HashSet};
use render::{
    mesh::Mesh, scheduled_pipeline::ShaderLabel, texture::Texture, vertex::MeshVertex,
    window_state::WindowState, winit::dpi::PhysicalSize,
};

use crate::assets::{image::load_image, obj::read_mesh_from_file};

pub mod image;
pub mod obj;

// This holds the path to a file that is to be loaded
#[derive(Clone, Debug, Eq, PartialEq)]
enum UnloadedAsset {
    Image(String),
    Mesh(String),
    Material(String, Vec<String>),
    Unknown,
}

impl UnloadedAsset {
    fn new(file_path: &str) -> Option<Self> {
        match file_path.split(".").last()? {
            "obj" => Some(Self::Mesh(file_path.to_owned())),
            "png" | "jpg" | "jpeg" => Some(Self::Image(file_path.to_owned())),
            _ => Some(Self::Unknown),
        }
    }

    fn get_name(&self) -> &str {
        match self {
            Self::Image(name) => name,
            Self::Material(name, _) => name,
            Self::Mesh(name) => name,
            Self::Unknown => panic!("cannot get name of unknown file type"),
        }
    }

    fn load(self, ws: &WindowState) -> Option<(PendingAsset, Vec<UnloadedAsset>)> {
        match self {
            Self::Image(path) => {
                let (data, (width, height)) = load_image(&path);
                let img = Texture::new(&path, PhysicalSize { width, height }, &ws.device);
                img.draw_image(&data[..], &ws.queue);

                Some((
                    PendingAsset {
                        name: path,
                        data: PendingAssetType::Image { img },
                    },
                    vec![],
                ))
            }
            Self::Material(path, imgs) => {
                let to_load = imgs
                    .iter()
                    .map(|path| Self::new(&path).expect("Material file references null path"))
                    .collect();

                Some((
                    PendingAsset {
                        name: path,
                        data: PendingAssetType::Material { textures: imgs },
                    },
                    to_load,
                ))
            }
            Self::Mesh(path) => {
                let (mesh, textures) = read_mesh_from_file(&path);

                Some((
                    PendingAsset {
                        name: path.clone(),
                        data: PendingAssetType::Mesh {
                            mesh,
                            material: path.clone(),
                        },
                    },
                    vec![UnloadedAsset::Material(path + ".mtl", textures)],
                ))
            }
            Self::Unknown => None,
        }
    }
}

// This holds data for a file while waiting for associated
// file to be loaded
enum PendingAssetType {
    Image {
        img: Texture,
    },
    Material {
        textures: Vec<String>,
    },
    Mesh {
        mesh: Mesh<MeshVertex>,
        material: String,
    },
}

struct PendingAsset {
    data: PendingAssetType,
    name: String,
}

impl PendingAsset {}

pub enum FinalizedAsset {
    Image {
        label: ShaderLabel,
        texture: Texture,
    },
    Material {},
    Mesh {},
}

// This is a struct that is responsible for keeping track of
// assets and loading only the required ammount.
// i.e. if the same mesh is loaded twice, its texture array is only needed
// once. When instances are implemented, then the mesh only needs to be loaded
// once too.
//
// AssetManager::new()
//      .load("mesh1")
//      .load("mesh2")
//      .load("mesh1") // won't actually load this fie
pub struct AssetManager {
    unloaded: VecDeque<UnloadedAsset>,
    // Which files has the manager been made aware of?
    // May or may not be loaded into a bucket
    present: HashSet<String>,
    obj_bucket: HashMap<String, (Mesh<MeshVertex>, String)>,
    mtl_bucket: HashMap<String, (ShaderLabel, Vec<String>)>,
    img_bucket: HashMap<String, ShaderLabel>,
    // default stuff in case of file read failures
    default_mesh: Option<Mesh<MeshVertex>>,
    default_img: Option<ShaderLabel>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            unloaded: VecDeque::new(),
            present: HashSet::new(),
            obj_bucket: HashMap::new(),
            mtl_bucket: HashMap::new(),
            img_bucket: HashMap::new(),
            default_mesh: None,
            default_img: None,
        }
    }

    pub fn with_default_mesh(mut self, default: Mesh<MeshVertex>) -> Self {
        self.default_mesh = Some(default);
        self
    }

    pub fn with_default_img(mut self, default: ShaderLabel) -> Self {
        self.default_img = Some(default);
        self
    }

    // Don't read this file now, but ho
    pub fn add(mut self, file_path: &str) -> Option<Self> {
        self.unloaded.push_front(UnloadedAsset::new(file_path)?);
        self.present.insert(file_path.to_owned());
        Some(self)
    }

    fn finalize_asset(&mut self, asset: PendingAsset) -> FinalizedAsset {
        match asset.data {
            PendingAssetType::Image { img } => todo!(),
            PendingAssetType::Material { textures } => todo!(),
            PendingAssetType::Mesh { mesh, material } => todo!(),
        }
    }

    // has this object already been loaded into a bucket?
    fn has_loaded_asset(&self, asset: &PendingAsset) -> bool {
        match asset.data {
            PendingAssetType::Image { .. } => self.img_bucket.contains_key(&asset.name),
            PendingAssetType::Material { .. } => self.mtl_bucket.contains_key(&asset.name),
            PendingAssetType::Mesh { .. } => self.obj_bucket.contains_key(&asset.name),
        }
    }

    // push a dependency file onto the unloaded stack.
    // does no verification
    fn push_dep(&mut self, asset: UnloadedAsset) {
        self.present.insert(asset.get_name().to_owned());
        self.unloaded.push_front(asset);
    }

    // start parsing and loading files into cache buckets
    pub fn read(&mut self, ws: &WindowState) -> Option<Vec<FinalizedAsset>> {
        let mut pending_stack = VecDeque::new();

        while !self.unloaded.is_empty() {
            let asset = self
                .unloaded
                .pop_front()
                .expect("Failed to pop existing entry from unloaded asset stack");

            let (pending, dependencies) = asset.load(ws)?;

            pending_stack.push_front(pending);
            for dep in dependencies {
                self.push_dep(dep);
            }
        }

        // since dependencies are pushed to top of stack, they must already
        // be loaded
        let mut finalized_assets = Vec::with_capacity(pending_stack.len());
        while !pending_stack.is_empty() {
            let asset = pending_stack
                .pop_front()
                .expect("Failed to pop existing entry from pending asset stack");
            if self.has_loaded_asset(&asset) {
                continue;
            }

            let finalized = self.finalize_asset(asset);
        }

        Some(finalized_assets)
    }

    // get a copy of a mesh from a bucket and an accessor
    // to the associated texture array bindgroup
    // return None if not in bucket
    pub fn load_mesh(&self, mesh_path: &str) {
        todo!()
    }

    // get some kind of reference to an image (either bindgroup or shaderlabel)
    // return None if not in bucket
    pub fn load_image(&mut self, img_path: &str) {
        todo!()
    }

    // get the material bind group (texture array) and corresponding
    // texture buffers
    // return None if not in bucket
    pub fn load_material(&mut self, mtl_path: &str) {
        todo!()
    }
}

unsafe impl FrostyAllocatable for AssetManager {}
