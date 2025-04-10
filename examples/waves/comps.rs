//
// The Scene objects are pretty simple:
//  1) A fly camera
//  2) A plane mesh
//  3) Some sun representation
//  4) Clouds!
//  5) A bird to sit on the water and fly around
//

use engine_core::SceneBuilder;
use frosty_alloc::{AllocId, FrostyAllocatable};

pub(crate) fn register_comps(scene: SceneBuilder) -> SceneBuilder {
    scene
        .register_component::<OceanMesh>()
        .register_component::<Sun>()
        .register_component::<Cloud>()
        .register_component::<Bird>()
}

struct OceanVertex {}

struct OceanMesh {}

struct Sun {}

struct Cloud {}

struct Bird {}

unsafe impl FrostyAllocatable for OceanMesh {}

unsafe impl FrostyAllocatable for Sun {}

unsafe impl FrostyAllocatable for Cloud {}

unsafe impl FrostyAllocatable for Bird {}
