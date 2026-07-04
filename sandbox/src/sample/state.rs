use bevy::prelude::*;

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
