use bevy::asset::{AssetPath, embedded_asset};
use bevy::platform::collections::HashMap;
use bevy::shader::ShaderRef;
use bevy::{
    asset::uuid::Uuid, color::palettes::css, pbr::ExtendedMaterial, prelude::*,
    render::render_resource::AsBindGroup,
};

use super::state::SampleModel;
use super::state::{SampleMaterialType, SampleState};
use crate::sample::emitter::AutoPlay;
use crate::sample::extended_material::{MyExtendedMaterial, MyExtension, ReloadReq};
use crate::sample::scene_mod::{TrailEmitter, TrailEmitterTiming};

pub fn init_custom_material(app: &mut App) {
    embedded_asset!(app, "vertex_for_custom.wgsl");
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct CustomMaterial {
    #[uniform(129)]
    spawned_at: f32,

    // NOTE: For now, we use `webgl2` feature flag (default) of bevy, it requires uniform buffer size to be 16 bytes aligned, so we add some padding here.
    #[uniform(129)]
    _weggl2_padding_8b: u32,
    #[uniform(129)]
    _weggl2_padding_12b: u32,
    #[uniform(129)]
    _weggl2_padding_16b: u32,
}

#[derive(Resource)]
pub struct CustomMaterialShader(pub String);

const CUSTOM_MATERIAL_WGSL_UUID: Uuid = Uuid::from_u128(0xffff0000aaaabdef1234567890abcdee);
const CUSTOM_MATERIAL_WGSL_PATH: &str = "globals:custom_material.wgsl";

pub fn request_load_custom_material(mut req_sender: MessageWriter<ReloadReq>) {
    req_sender.write(ReloadReq);
}

pub fn load_custom_material(
    mut shaders: ResMut<Assets<Shader>>,
    mut reload_reqs: MessageReader<ReloadReq>,
    custom_shader: Res<CustomMaterialShader>,
) {
    let is_requested = reload_reqs.read().any(|_| true);
    if !is_requested {
        return;
    }
    let shader = Shader::from_wgsl(
        custom_shader.0.clone(),
        CUSTOM_MATERIAL_WGSL_PATH.to_string(),
    );
    shaders
        .insert(CUSTOM_MATERIAL_WGSL_UUID, shader)
        .expect("Failed to insert shader");
    info!("Reloaded custom material shader");
}

impl Material for CustomMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        ShaderRef::Path(AssetPath::from(
            "embedded://sandbox/sample/vertex_for_custom.wgsl",
        ))
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        bevy::shader::ShaderRef::Handle(CUSTOM_MATERIAL_WGSL_UUID.into())
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    // FIXME: duplicated with ExtendedMaterial
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::material::descriptor::RenderPipelineDescriptor,
        layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::material::specialize::SpecializedMeshPipelineError> {
        assert!(
            descriptor.vertex.buffers.len() == 1,
            "Expected only one vertex buffer layout for the mesh"
        );

        // WORKAROUND
        let is_prepass_pipeline = descriptor
            .label
            .as_ref()
            .is_some_and(|s| s == "prepass_pipeline");

        // Mesh attributes for Bevy PBR
        // constructed in `specialize()` of `SpecializedMeshPipeline` for `MeshPipeline`
        // https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/mesh.rs
        let base_mesh_attirbutes_list_for_render = [
            // in specialize()
            (Mesh::ATTRIBUTE_POSITION, 0),
            (Mesh::ATTRIBUTE_NORMAL, 1),
            (Mesh::ATTRIBUTE_UV_0, 2),
            (Mesh::ATTRIBUTE_UV_1, 3),
            (Mesh::ATTRIBUTE_TANGENT, 4),
            (Mesh::ATTRIBUTE_COLOR, 5),
            // in setup_morph_and_skinning_defs()
            //   https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/mesh.rs
            (Mesh::ATTRIBUTE_JOINT_INDEX, 6),
            (Mesh::ATTRIBUTE_JOINT_WEIGHT, 7),
        ];
        let base_mesh_attributes_list_for_prepass = [
            // see bevy_pbr/src/prepass/mod.rs
            (Mesh::ATTRIBUTE_POSITION, 0),
            (Mesh::ATTRIBUTE_UV_0, 1),
            (Mesh::ATTRIBUTE_UV_1, 2),
            (Mesh::ATTRIBUTE_NORMAL, 3),
            (Mesh::ATTRIBUTE_TANGENT, 4),
            (Mesh::ATTRIBUTE_COLOR, 7),
            // in setup_morph_and_skinning_defs
            (Mesh::ATTRIBUTE_JOINT_INDEX, 5),
            (Mesh::ATTRIBUTE_JOINT_WEIGHT, 6),
        ];

        let base_mesh_attirbutes_list = if is_prepass_pipeline {
            &base_mesh_attributes_list_for_prepass
        } else {
            &base_mesh_attirbutes_list_for_render
        };

        let mut vertex_attriutes = Vec::new();
        // reconstruct the vertex attributes list for the mesh.
        for (attr, loc) in base_mesh_attirbutes_list {
            if layout.0.contains(*attr) {
                vertex_attriutes.push(attr.at_shader_location(*loc));
            }
        }

        if layout.0.contains(my_meshes::ATTRIBUTE_TIME) && !is_prepass_pipeline {
            vertex_attriutes.push(my_meshes::ATTRIBUTE_TIME.at_shader_location(10));
            descriptor
                .vertex
                .shader_defs
                .push("MY_MESHES_ATTRIBUTE_TIME".into());
            if let Some(shader_defs) = descriptor.fragment.as_mut().map(|f| &mut f.shader_defs) {
                shader_defs.push("MY_MESHES_ATTRIBUTE_TIME".into());
            }
            info!(
                "Found my_meshes::ATTRIBUTE_TIME in mesh layout, adding shader def MY_MESHES_ATTRIBUTE_TIME"
            );
        } else {
            info!(
                "my_meshes::ATTRIBUTE_TIME not found in mesh layout, shader def MY_MESHES_ATTRIBUTE_TIME not added"
            );
        }

        let vertex_layout = layout.0.get_layout(&vertex_attriutes)?;
        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }

    // fn enable_prepass() -> bool {
    //     false
    // }
}

#[allow(clippy::too_many_arguments)]
pub fn insert_sample_material(
    mut commands: Commands,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut extended_materials: ResMut<Assets<MyExtendedMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    sample_state: Res<SampleState>,
    q_sample_models: Query<(Entity, &SampleModel), Added<SampleModel>>,
    time: Res<Time>,
) {
    for (entity, model) in q_sample_models.iter() {
        if *model != SampleModel::Mesh {
            continue;
        }
        // add material
        match sample_state.material_type {
            SampleMaterialType::CustomMaterial => {
                commands
                    .entity(entity)
                    .try_insert(MeshMaterial3d(custom_materials.add(CustomMaterial {
                        spawned_at: time.elapsed_secs(),
                        ..default()
                    })));
            }
            SampleMaterialType::ExtendedMaterial => {
                let base_material = StandardMaterial {
                    base_color: css::LIGHT_GRAY.into(),
                    alpha_mode: AlphaMode::Blend, // TODO:
                    ..Default::default()
                };
                let material = ExtendedMaterial {
                    base: base_material,
                    extension: MyExtension::new(
                        LinearRgba::new(1.0, 0.0, 0.0, 0.25),
                        time.elapsed_secs(),
                    ),
                };
                commands
                    .entity(entity)
                    .try_insert(MeshMaterial3d(extended_materials.add(material)));
            }
            SampleMaterialType::UvTexture => {
                let texture_handle = asset_server.load(myshaderlib::path_to_uv_test1024());
                let material = StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    ..Default::default()
                };
                commands
                    .entity(entity)
                    .try_insert(MeshMaterial3d(standard_materials.add(material)));
            }
        }
    }
}

#[derive(Component, Reflect, Default, serde::Deserialize, Debug, Clone)]
#[reflect(Component)]
pub struct SandboxExtension {
    pub shader_type: String,
    pub param1: [f32; 4],
}

const EXTENSION_MATERIAL_NAME: &str = "ASHIOJIN_material_sandbox";

#[derive(Component, Reflect, Default, serde::Deserialize, Debug, Clone)]
#[reflect(Component)]
pub struct SandboxMeshFxConfigExtension {
    pub is_fx_mesh: bool,
    pub fx_type: String,
}
const EXTENSION_MESH_FX_CONFIG_NAME: &str = "ASHIOJIN_mesh_fx_config";

const EXTENSION_ANIMATION_FX_CONFIGS_NAME: &str = "ASHIOJIN_action_fx_config";

#[derive(Component, Reflect, Default, serde::Deserialize, Debug, Clone)]
#[reflect(Component)]
pub struct SandboxActionFxConfigExtension {
    fx_configs: Vec<FxConfig>,
}
#[derive(Reflect, Default, serde::Deserialize, Debug, Clone)]
pub struct FxConfig {
    pub target_name: String,
    pub start_sec: f32,
    pub end_sec: f32,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct SandboxActionFxConfig {
    pub maps: HashMap<String, Vec<FxConfig>>, // Animation(Action) name -> Vec<FxConfig>
}

#[derive(Default, Clone)]
pub struct ReplaceMaterialGltfExtensionHandler {
    animation_fx_configs: SandboxActionFxConfig,
}

impl bevy::gltf::extensions::GltfExtensionHandler for ReplaceMaterialGltfExtensionHandler {
    fn dyn_clone(&self) -> Box<dyn bevy::gltf::extensions::ErasedGltfExtensionHandler> {
        Box::new(self.clone())
    }

    fn on_spawn_mesh_and_material(
        &mut self,
        _load_context: &mut bevy::asset::LoadContext<'_>,
        _primitive: &gltf::Primitive,
        mesh: &gltf::Mesh,
        material: &gltf::Material,
        entity: &mut EntityWorldMut,
        _material_label: &str,
    ) {
        if let Some(extension_value) = material.extension_value(EXTENSION_MATERIAL_NAME) {
            let sandbox_extension: SandboxExtension =
                serde_json::from_value(extension_value.clone())
                    .expect("Failed to parse ASHIOJIN_material_sandbox extension");

            if sandbox_extension.shader_type == "ASHIOJIN_SANDBOX" {
                entity.insert(sandbox_extension);

                let t = entity.get_resource::<Assets<StandardMaterial>>().is_some();
                debug!("{t:?}");
            }
        }
        if let Some(extension_value) = mesh.extension_value(EXTENSION_MESH_FX_CONFIG_NAME) {
            let mesh_fx_config_extension: SandboxMeshFxConfigExtension =
                serde_json::from_value(extension_value.clone())
                    .expect("Failed to parse ASHIOJIN_mesh_fx_config extension");
            entity.insert(mesh_fx_config_extension);
        }

        debug!("Mesh {:?}, Ext: {:?}", mesh.name(), mesh.extensions());
        debug!(
            "Material {:?}, Ext: {:?}",
            material.name(),
            material.extensions()
        );
    }

    fn on_animation(
        &mut self,
        _load_context: &mut bevy::asset::LoadContext<'_>,
        gltf_animation: &gltf::Animation,
        _animation_clip: &mut AnimationClip,
    ) {
        if let Some(extension_value) =
            gltf_animation.extension_value(EXTENSION_ANIMATION_FX_CONFIGS_NAME)
        {
            info!(
                "Animation {:?} has extension {:?} = {:?}",
                gltf_animation.name(),
                EXTENSION_ANIMATION_FX_CONFIGS_NAME,
                extension_value
            );
            let fx_config_extenion: SandboxActionFxConfigExtension = serde_json::from_value(extension_value.clone())
                .expect("Failed to parse ASHIOJIN_animation_fx_config extension");
            self.animation_fx_configs.maps.insert(
                gltf_animation.name().unwrap_or_default().to_string(),
                fx_config_extenion.fx_configs
            );
        }
    }

    fn on_scene_completed(
        &mut self,
        _load_context: &mut bevy::asset::LoadContext<'_>,
        scene: &gltf::Scene,
        world_root_id: Entity,
        scene_world: &mut World,
    ) {
        // add SandboxActionFxConfig component to the root entity of the scene
        info!(
            "Scene {:?} completed, adding SandboxActionFxConfig to root entity {:?}",
            scene.name(),
            world_root_id
        );
        if !self.animation_fx_configs.maps.is_empty() {
            if let Ok(mut root_entity) = scene_world.get_entity_mut(world_root_id) {
                root_entity.insert(SandboxActionFxConfig {
                    maps: self.animation_fx_configs.maps.clone(),
                });
                info!(
                    "Added SandboxActionFxConfig to root entity {:?} with maps: {:?}",
                    world_root_id,
                    self.animation_fx_configs.maps.keys().collect::<Vec<_>>()
                );
            } else {
                warn!("Root entity not found for scene {:?}", scene.name());
            }
        }
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn apply_sandbox_materials(
    mut commands: Commands,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut extended_materials: ResMut<Assets<MyExtendedMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    sample_state: Res<SampleState>,
    time: Res<Time>,
    query: Query<
        (
            Entity,
            &SandboxExtension,
            Option<&MeshMaterial3d<StandardMaterial>>,
        ),
        Added<SandboxExtension>,
    >,
) {
    for (entity, sandbox_extension, standard_material_handle) in query.iter() {
        let standard_material = standard_material_handle
            .and_then(|handle| standard_materials.get(&handle.0))
            .cloned();

        // 2. add a custom material according to the sample state
        let material_type = sample_state.material_type;
        match material_type {
            SampleMaterialType::CustomMaterial => {
                let custom_material = CustomMaterial {
                    spawned_at: time.elapsed_secs(),
                    ..default()
                };
                let asset_handle = custom_materials.add(custom_material);
                commands
                    .entity(entity)
                    .try_insert(MeshMaterial3d(asset_handle));
            }
            SampleMaterialType::ExtendedMaterial => {
                let base_material = standard_material.unwrap_or(StandardMaterial {
                    base_color: css::LIGHT_GRAY.into(),
                    alpha_mode: AlphaMode::Blend,
                    ..Default::default()
                });
                let material = ExtendedMaterial {
                    base: base_material,
                    extension: MyExtension::new(
                        LinearRgba::from_f32_array(sandbox_extension.param1),
                        time.elapsed_secs(),
                    ),
                };
                let asset_handle = extended_materials.add(material);
                commands
                    .entity(entity)
                    .try_insert(MeshMaterial3d(asset_handle));
            }
            SampleMaterialType::UvTexture => {
                let texture_handle = asset_server.load(myshaderlib::path_to_uv_test1024());
                let material = StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    ..Default::default()
                };
                let asset_handle = standard_materials.add(material);
                commands
                    .entity(entity)
                    .try_insert(MeshMaterial3d(asset_handle));
            }
        }
    }
}

pub fn apply_sandbox_fx_meshes(
    mut commands: Commands,
    _time: Res<Time>,
    #[allow(clippy::type_complexity)] query: Query<
        (
            Entity,
            &SandboxMeshFxConfigExtension,
            &Name,
            Option<&Mesh3d>,
        ),
        With<SandboxMeshFxConfigExtension>,
    >,
    q_scene_root: Query<(Entity, &AutoPlay), Added<AutoPlay>>,
    q_fx_cocnfig: Query<(Entity, &SandboxActionFxConfig)>,
    q_children: Query<&Children>,
) {
    // Add TrailEmitter
    for (root_entity, auto_play) in q_scene_root.iter() {
        let Some((_, fx_config)) = q_children.iter_descendants(root_entity).find_map(|entity| {
            q_fx_cocnfig.get(entity).ok()
        }) else {
            info!(
                "Scene root entity {:?} has AutoPlay but no SandboxActionFxConfig",
                root_entity
            );
            continue;
        };

        info!(
            "Scene root entity {:?} has AutoPlay and SandboxActionFxConfig with maps: {:?}",
            root_entity,
            fx_config.maps.keys().collect::<Vec<_>>()
        );
        q_children.iter_descendants(root_entity).for_each(|entity| {
            if let Ok((_, mesh_fx_config_extension, name, _mesh3d)) =
                query.get(entity)
            {
                if !mesh_fx_config_extension.is_fx_mesh {
                    info!(
                        "Skipping entity {:?} (name: {:?}) because is_fx_mesh is false",
                        entity,
                        name.as_str()
                    );
                    return;
                }

                let mut timings = vec![];
                for (action_name, fx_configs) in fx_config.maps.iter() {
                    let Some(anim_node_idx) = auto_play
                        .node_idx_list()
                        .iter()
                        .find(|(n, _)| n.as_str() == action_name.as_str())
                        .map(|(_, idx)| idx)
                    else {
                        continue;
                    };
                    let fx_configs = fx_configs
                        .iter()
                        .filter(|fx| fx.target_name == name.as_str())
                        .map(|fx| TrailEmitterTiming::new(*anim_node_idx, fx.start_sec, fx.end_sec));

                    timings.extend(fx_configs);
                }

                info!(
                    "-- node_idx_list: {:?}, fx_config: {:?}, entity: {:?}, name: {:?}, timings: {:?}",
                    auto_play.node_idx_list(),
                    fx_config,
                    entity,
                    name.as_str(),
                    timings
                );

                info!(
                    "Adding TrailEmitter to entity {:?} (name: {:?}) with timings: {:?}",
                    entity,
                    name.as_str(),
                    timings
                );

                commands.entity(entity).try_insert((
                    TrailEmitter::new(0.2).extend_timings(timings),
                    Visibility::Hidden,
                ));
            }
        });

    }



    // TODO: Should use `fx_type` to determine which effect to apply. For now, we only have one effect, so we ignore it.
    // for (entity, mesh_fx_config_extension, name, _mesh3d) in query.iter() {
    //     if !mesh_fx_config_extension.is_fx_mesh {
    //         info!(
    //             "Skipping entity {:?} (name: {:?}) because is_fx_mesh is false",
    //             entity,
    //             name.as_str()
    //         );
    //         continue;
    //     }
    //
    //     let Some((_root_entity, auto_play, op_fx_config)) = q_children
    //         .iter_ancestors(entity)
    //         .find_map(|e| q_scene_root.get(e).ok())
    //     else {
    //         info!(
    //             "Skipping entity {:?} (name: {:?}) because it is not a child of a scene root with AutoPlay and SandboxActionFxConfig",
    //             entity,
    //             name.as_str()
    //         );
    //         continue;
    //     };
    //
    //     let Some(fx_config) = op_fx_config else {
    //         info!(
    //             "Skipping entity {:?} (name: {:?}) because it is not a child of a scene root with SandboxActionFxConfig",
    //             entity,
    //             name.as_str()
    //         );
    //         continue;
    //     };
    //
    //     let mut timings = vec![];
    //     for (action_name, fx_configs) in fx_config.maps.iter() {
    //         let Some(anim_node_idx) = auto_play
    //             .node_idx_list()
    //             .iter()
    //             .find(|(n, _)| n.as_str() == action_name.as_str())
    //             .map(|(_, idx)| idx)
    //         else {
    //             continue;
    //         };
    //         let fx_configs = fx_configs
    //             .iter()
    //             .filter(|fx| fx.target_name == name.as_str())
    //             .map(|fx| TrailEmitterTiming::new(*anim_node_idx, fx.start_sec, fx.end_sec));
    //
    //         timings.extend(fx_configs);
    //     }
    //
    //     info!(
    //         "Adding TrailEmitter to entity {:?} (name: {:?}) with timings: {:?}",
    //         entity,
    //         name.as_str(),
    //         timings
    //     );
    //
    //     commands.entity(entity).try_insert((
    //         TrailEmitter::new(0.2).extend_timings(timings),
    //         // TODO: Informations to define an effect emitter and animation clips & graph
    //         // should be included .gltf or something asset file including .gltf file path(es),
    //         // and then should be used to construct required Components.
    //         // Currenty, Our blender plugin does not have features to define & export these infomrations. So we just hardcode them here for now.
    //         Visibility::Hidden,
    //     ))
    //         ;
    // }
}
