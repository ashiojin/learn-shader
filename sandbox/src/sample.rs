mod state;
mod mesh;
mod material;
mod extended_material;
mod emitter;

pub use state::SampleState;
pub use mesh::{SampleMesh, refresh_sample_mesh};
pub use material::{CustomMaterial, insert_sample_material, reload_shaders};
pub use extended_material::MyExtendedMaterialPlugin;
pub use emitter::{despawn_expired, spawn_mesh_from_emitter};
