use bevy::prelude::*;
use super::mesh::SampleType;
use super::material::SampleMaterialType;

#[derive(Resource, Debug, Default)]
pub struct SampleState {
    pub sample_type: SampleType,
    pub material_type: SampleMaterialType,
}

impl SampleState {
    pub fn next_sample(&mut self) {
        self.sample_type = self.sample_type.get_next();
    }
    pub fn next_material(&mut self) {
        self.material_type = self.material_type.get_next();
    }

    pub fn spawn(&self, commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>) {
        self.sample_type.spawn(commands, meshes);
    }
}
