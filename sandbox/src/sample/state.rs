use bevy::prelude::*;

/// Describes a GLTF asset backing a [`SampleType`] variant.
/// Single source of truth for the sandbox's GLTF file paths, consumed by both
/// the preloader and `spawn_sample`.
#[derive(Debug, Clone, Copy)]
pub struct GltfSampleAsset {
    pub path: &'static str,
    pub scene_idx: usize,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum SampleType {
    #[default]
    Saru,
    ArmAndRod,
    Plane,
    Cube,
    Cone,
    Sphere,
    Ring,
    SphericalZone,
    Belt,
    Emitter1,
}

impl SampleType {
    pub const fn all_variants() -> &'static [Self] {
        &[
            Self::Saru,
            Self::ArmAndRod,
            Self::Plane,
            Self::Cube,
            Self::Cone,
            Self::Sphere,
            Self::Ring,
            Self::SphericalZone,
            Self::Belt,
            Self::Emitter1,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Saru => "Saru",
            Self::ArmAndRod => "ArmAndRod",
            Self::Plane => "Plane",
            Self::Cube => "Cube",
            Self::Cone => "Cone",
            Self::Sphere => "Sphere",
            Self::Ring => "Ring",
            Self::SphericalZone => "SphericalZone",
            Self::Belt => "Belt",
            Self::Emitter1 => "Emitter1",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Self::all_variants().iter().find(|v| v.as_str() == s).cloned()
    }

    pub fn get_next(&self) -> Self {
        let variants = Self::all_variants();
        let idx = variants.iter().position(|v| v == self).unwrap_or(0);
        variants[(idx + 1) % variants.len()].clone()
    }

    /// Returns the GLTF asset descriptor for GLTF-backed variants, else `None`.
    pub fn gltf_asset(&self) -> Option<GltfSampleAsset> {
        match self {
            Self::Saru => Some(GltfSampleAsset {
                path: "models/saru.glb",
                scene_idx: 0,
            }),
            Self::ArmAndRod => Some(GltfSampleAsset {
                path: "models/poc_arm_and_rod_ex.glb",
                scene_idx: 0,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq, Clone, Copy)]
pub enum SampleMaterialType {
    #[default]
    CustomMaterial,
    ExtendedMaterial,
    UvTexture,
}

impl SampleMaterialType {
    pub const fn all_variants() -> &'static [Self] {
        &[
            Self::CustomMaterial,
            Self::ExtendedMaterial,
            Self::UvTexture,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CustomMaterial => "CustomMaterial",
            Self::ExtendedMaterial => "ExtendedMaterial",
            Self::UvTexture => "UvTexture",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Self::all_variants().iter().find(|v| v.as_str() == s).cloned()
    }

    pub fn get_next(&self) -> Self {
        let variants = Self::all_variants();
        let idx = variants.iter().position(|v| v == self).unwrap_or(0);
        variants[(idx + 1) % variants.len()]
    }
}

#[derive(Resource, Debug, Default)]
pub struct SampleState {
    pub sample_type: SampleType,
    pub material_type: SampleMaterialType,
    pub model_billboard: bool,
}

impl SampleState {
    pub fn next_sample(&mut self) {
        self.sample_type = self.sample_type.get_next();
    }
    pub fn next_material(&mut self) {
        self.material_type = self.material_type.get_next();
    }
    pub fn toggle_billboard(&mut self) {
        self.model_billboard = !self.model_billboard;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleModel {
    Mesh,
    Scene,
}

#[derive(Component, Debug)]
pub struct SampleEmitter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gltf_asset_matches_known_variants() {
        // GLTF-backed variants expose a non-empty path.
        let saru = SampleType::Saru.gltf_asset().expect("Saru is GLTF-backed");
        assert_eq!(saru.path, "models/saru.glb");
        assert_eq!(saru.scene_idx, 0);

        let arm = SampleType::ArmAndRod
            .gltf_asset()
            .expect("ArmAndRod is GLTF-backed");
        assert_eq!(arm.path, "models/poc_arm_and_rod_ex.glb");
        assert_eq!(arm.scene_idx, 0);

        // Non-GLTF variants return None.
        assert!(SampleType::Cube.gltf_asset().is_none());
        assert!(SampleType::Plane.gltf_asset().is_none());
        assert!(SampleType::Emitter1.gltf_asset().is_none());
    }

    #[test]
    fn every_gltf_asset_path_is_non_empty() {
        for variant in SampleType::all_variants() {
            if let Some(asset) = variant.gltf_asset() {
                assert!(
                    !asset.path.is_empty(),
                    "{:?} has an empty GLTF path",
                    variant
                );
            }
        }
    }
}
