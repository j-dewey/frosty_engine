use core::panic;
//
// functions for useful asset management
//
use std::{collections::VecDeque, sync::Arc};

use frosty_alloc::FrostyAllocatable;
use hashbrown::{HashMap, HashSet};
use render::{
    mesh::Mesh, scheduled_pipeline::ShaderLabel, shader::Shader, texture::Texture,
    vertex::MeshVertex, window_state::GPUBindings, winit::dpi::PhysicalSize,
};

use crate::{
    assets::{image::load_image, obj::read_mesh_from_file},
    query::Query,
    system::{System, UpdateResult},
};

pub mod image;
pub mod obj;
pub mod pending;

/*
 * How are assets stored?
 *      Mesh =>
 *      Material =>
 *      Image =>
 */

pub type AssetLabel = ShaderLabel;

#[derive(Clone, Debug)]
pub struct Asset {
    data: UnloadedAsset,
    label: AssetLabel,
}
unsafe impl FrostyAllocatable for Asset {}

// This holds the path to a file that is to be loaded
#[derive(Clone, Debug, Eq, PartialEq)]
enum UnloadedAsset {
    Image(String),
    Mesh(String),
    Material(ShaderLabel, Vec<String>),
    Unknown,
}

impl UnloadedAsset {
    fn new(file_path: String) -> Option<Self> {
        match file_path.split(".").last()? {
            "obj" => Some(Self::Mesh(file_path)),
            "png" | "jpg" | "jpeg" => Some(Self::Image(file_path)),
            _ => Some(Self::Unknown),
        }
    }

    fn get_name(&self) -> &str {
        match self {
            Self::Image(name) => name,
            Self::Material(name, _) => name.label(),
            Self::Mesh(name) => name,
            Self::Unknown => panic!("cannot get name of unknown file type"),
        }
    }

    fn load(self, res_path: &str, gpu: &GPUBindings) -> Option<(PendingAsset, Vec<UnloadedAsset>)> {
        match self {
            Self::Image(path) => {
                let (data, (width, height)) = load_image(&path, res_path);
                let texture = Texture::new(&path, PhysicalSize { width, height }, &gpu.device);
                texture.draw_image(&data[..], &gpu.queue);

                Some((
                    PendingAsset {
                        name: AssetLabel::Dynamic(Arc::new(path)),
                        data: PendingAssetType::Image { texture },
                    },
                    vec![],
                ))
            }
            Self::Material(label, imgs) => {
                let mut images = Vec::with_capacity(imgs.len());
                let to_load = imgs
                    .iter()
                    .map(|path| {
                        let full_path = res_path.to_owned() + path;
                        images.push(AssetLabel::Dynamic(Arc::new(full_path.clone())));
                        Self::new(full_path).expect("Material file references null path")
                    })
                    .collect();

                Some((
                    PendingAsset {
                        name: label,
                        data: PendingAssetType::Material { textures: images },
                    },
                    to_load,
                ))
            }
            Self::Mesh(path) => {
                let (mesh, mat_label, textures) = read_mesh_from_file(&path, res_path);

                Some((
                    PendingAsset {
                        name: ShaderLabel::Dynamic(Arc::new(path)),
                        data: PendingAssetType::Mesh {
                            mesh,
                            material: mat_label.clone(),
                        },
                    },
                    vec![UnloadedAsset::Material(mat_label, textures)],
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
        texture: Texture,
    },
    Material {
        textures: Vec<AssetLabel>,
    },
    Mesh {
        mesh: Mesh<MeshVertex>,
        material: ShaderLabel,
    },
}

struct PendingAsset {
    data: PendingAssetType,
    name: AssetLabel,
}

impl PendingAsset {}

pub enum FinalizedAsset {
    // This ends as a Label and a Texture
    Image {
        label: AssetLabel,
        texture: Texture,
    },
    // This ends as a vector of Labels corresponding to all
    // the textures held in it
    Material {
        label: AssetLabel,
        textures: Vec<ShaderLabel>,
    },
    // This ends as a mesh and a vector of Labels corresponding to all
    // the textures it depends on
    Mesh {
        mesh: Mesh<MeshVertex>,
        label: AssetLabel,
    },
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
    // This is the path to the folder that stores all the assets
    resource_path: String,
    unloaded: VecDeque<UnloadedAsset>,
    // Which files has the manager been made aware of?
    // May or may not be loaded into a bucket
    present: HashSet<String>,
    obj_bucket: HashMap<ShaderLabel, (Mesh<MeshVertex>, ShaderLabel)>,
    mtl_bucket: HashMap<ShaderLabel, Vec<ShaderLabel>>,
    img_bucket: HashMap<ShaderLabel, ShaderLabel>,
    // default stuff in case of file read failures
    default_mesh: Option<Mesh<MeshVertex>>,
    default_img: Option<ShaderLabel>,
}

impl AssetManager {
    pub fn new(path: String) -> Self {
        Self {
            resource_path: path,
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

    // Don't read this file now, but hold onto it for later
    pub fn add(mut self, file_path: &str) -> Option<Self> {
        self.unloaded
            .push_front(UnloadedAsset::new(file_path.to_owned())?);
        self.present.insert(file_path.to_owned());
        Some(self)
    }

    fn finalize_asset(&mut self, asset: PendingAsset) -> FinalizedAsset {
        match asset.data {
            PendingAssetType::Image { texture } => {
                let label = asset.name.clone();
                self.img_bucket.insert(asset.name.clone(), label.clone());
                FinalizedAsset::Image { label, texture }
            }
            PendingAssetType::Material { textures } => {
                let held_textures: Vec<ShaderLabel> = textures
                    .iter()
                    .map(|name| {
                        self.img_bucket
                            .get(name)
                            .or(self.default_img.as_ref())
                            .expect(
                                "Failed to load image from material file and no default image set",
                            )
                            .clone()
                    })
                    .collect();
                self.mtl_bucket
                    .insert(asset.name.clone(), held_textures.clone());

                FinalizedAsset::Material {
                    label: asset.name,
                    textures: held_textures,
                }
            }
            PendingAssetType::Mesh { mesh, material } => {
                self.obj_bucket
                    .insert(asset.name.clone(), (mesh.clone(), material.clone()));

                FinalizedAsset::Mesh {
                    mesh,
                    label: material,
                }
            }
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
    // does no verification. assumes path was fixed by parent asset
    fn push_dep(&mut self, asset: UnloadedAsset) {
        self.present.insert(asset.get_name().to_owned());
        self.unloaded.push_front(asset);
    }

    // start parsing and loading files into cache buckets
    pub fn read(&mut self, gpu: &GPUBindings) -> Option<Vec<FinalizedAsset>> {
        let mut pending_stack = VecDeque::new();

        while !self.unloaded.is_empty() {
            let asset = self
                .unloaded
                .pop_front()
                .expect("Failed to pop existing entry from unloaded asset stack");

            let (pending, dependencies) = asset.load(&self.resource_path, gpu)?;

            pending_stack.push_front(pending);
            for dep in dependencies {
                self.push_dep(dep);
            }
        }

        // since dependencies are pushed to top of stack, they will be
        // loaded before the original asset is reached
        let mut finalized_assets = Vec::with_capacity(pending_stack.len());
        while !pending_stack.is_empty() {
            let asset = pending_stack
                .pop_front()
                .expect("Failed to pop existing entry from pending asset stack");
            if self.has_loaded_asset(&asset) {
                continue;
            }

            let finalized = self.finalize_asset(asset);

            finalized_assets.push(finalized);
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

    // Load all assets of type C into self.unloaded
    pub fn read_query<
        C: FrostyAllocatable + pending::AssetFile<Output = O>,
        O: FrostyAllocatable,
    >(
        &mut self,
        mut query: Query<C>,
        thread: u32,
    ) {
        while let Some(asset) = query.next(thread) {}
    }
}

unsafe impl FrostyAllocatable for AssetManager {}

impl System for AssetManager {
    type Interop = Asset;

    fn update(&self, mut objs: Query<Self::Interop>, thread: u32) -> UpdateResult {
        /*
         * Want to inject loaded data into Entity
         * i.e if Entity contains UnloadedAsset::Mesh,
         *      Add Mesh to Entity, or load MeshData into existing Mesh
         *
         */
        while let Some(mut asset) = objs.next(thread) {
            let asset_ref = asset.as_mut();
            match &asset_ref.data {
                UnloadedAsset::Image(name) => {}
                UnloadedAsset::Material(lbl, mtls) => {}
                UnloadedAsset::Mesh(path) => {}
                UnloadedAsset::Unknown => panic!("Attempting to load unknown asset!"),
            }
        }
        UpdateResult::Skip
    }
}
