use std::f32::consts::PI;

use bevy::{prelude::*, render::render_resource::AsBindGroup};

use crate::{SHADER_ASSET_PATH, meshes};

#[derive(Component, Debug, Clone)]
pub struct SampleMesh;

#[derive(Debug, Clone, Default)]
pub enum SampleType {
    #[default]
    Plane,
    Cube,
    Cone,
    Sphere,
    Ring,
    SphericalZone,
}
impl SampleType {
    pub fn get_next(&self) -> Self {
        match self {
            // TODO: use a crate to generate this boilerplate code
            SampleType::Plane => SampleType::Cube,
            SampleType::Cube => SampleType::Cone,
            SampleType::Cone => SampleType::Sphere,
            SampleType::Sphere => SampleType::Ring,
            SampleType::Ring => SampleType::SphericalZone,
            SampleType::SphericalZone => SampleType::Plane,
        }
    }

    pub fn mesh(&self) -> Mesh {
        match self {
            SampleType::Plane => Plane3d::new(Vec3::Z, Vec2::new(1.0, 1.0)).into(),
            SampleType::Cube => Cuboid::from_length(1.0).mesh().into(),
            SampleType::Cone => Cone::new(0.5, 1.0).mesh().into(),
            SampleType::Sphere => Sphere::new(0.5).mesh().into(),
            SampleType::Ring => meshes::FlatRing3d::new(Dir3::Z, 1.0, 0.25)
                .with_resolution(32)
                .mesh().into(),
            SampleType::SphericalZone => meshes::SphericalZone::new(0.5, 7. * PI / 16.0, 9. * PI / 16.0)
                .with_circle_resolution(64)
                .with_angle_resolution(8)
                .with_double_sided(true)
                .mesh().into(),
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct SampleState {
    sample_type: SampleType,
    entity: Option<Entity>,

    material_type: SampleMaterialType,
}

#[derive(Debug, Default, Eq, PartialEq)]
pub enum SampleMaterialType {
    #[default]
    User,
    UvTest1024,
}

impl SampleState {
    pub fn next_sample(&mut self) {
        self.sample_type = self.sample_type.get_next();
    }
    pub fn next_material(&mut self) {
        self.material_type = match self.material_type {
            SampleMaterialType::User => SampleMaterialType::UvTest1024,
            SampleMaterialType::UvTest1024 => SampleMaterialType::User,
        };
    }
}

pub fn change_sample(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut sample_state: ResMut<SampleState>,
) {
    // 1. despawn old sample
    // 2. spawn new sample with new material
    if let Some(entity) = sample_state.entity {
        commands.entity(entity).despawn();
    }

    let entity = commands.spawn(
        (
            Mesh3d(meshes.add(sample_state.sample_type.mesh())),
            Transform::from_xyz(0., 0., 0.),
            SampleMesh,
        )
    ).id();

    // add material
    match sample_state.material_type {
        SampleMaterialType::User => {
            commands
                .entity(entity)
                .insert(MeshMaterial3d(materials.add(CustomMaterial {})));
        }
        SampleMaterialType::UvTest1024 => {
            let texture_handle = asset_server.load(myshaderlib::path_to_uv_test1024());
            let material = StandardMaterial {
                base_color_texture: Some(texture_handle),
                ..Default::default()
            };
            commands
                .entity(entity)
                .insert(MeshMaterial3d(standard_materials.add(material)));
        }
    }

    sample_state.entity = Some(entity);
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {}

impl Material for CustomMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

