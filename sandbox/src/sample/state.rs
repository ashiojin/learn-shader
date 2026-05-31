use bevy::prelude::*;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum SampleType {
    #[default]
    Saru,
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
    pub fn get_next(&self) -> Self {
        match self {
            SampleType::Saru => SampleType::Plane,
            SampleType::Plane => SampleType::Cube,
            SampleType::Cube => SampleType::Cone,
            SampleType::Cone => SampleType::Sphere,
            SampleType::Sphere => SampleType::Ring,
            SampleType::Ring => SampleType::SphericalZone,
            SampleType::SphericalZone => SampleType::Belt,
            SampleType::Belt => SampleType::Emitter1,
            SampleType::Emitter1 => SampleType::Saru,
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq, Clone, Copy)]
pub enum SampleMaterialType {
    #[default]
    User,
    UserExtended,
    UvTest1024,
}

impl SampleMaterialType {
    pub fn get_next(&self) -> Self {
        match self {
            SampleMaterialType::User => SampleMaterialType::UvTest1024,
            SampleMaterialType::UvTest1024 => SampleMaterialType::UserExtended,
            SampleMaterialType::UserExtended => SampleMaterialType::User,
        }
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
