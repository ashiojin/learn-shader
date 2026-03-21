use std::ops::Range;

use bevy::{
    camera::{MainPassResolutionOverride, Viewport},
    core_pipeline::core_3d::{CORE_3D_DEPTH_FORMAT, graph::{Core3d, Node3d}},
    ecs::{
        query::QueryItem,
        system::{
            SystemParamItem,
            lifetimeless::{Read, SRes},
        },
    },
    math::FloatOrd,
    pbr::{
        DrawMesh, MeshInputUniform, MeshPipeline, MeshPipelineKey, MeshPipelineViewLayoutKey,
        MeshUniform, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
        SetMeshViewEmptyBindGroup,
    },
    platform::collections::HashSet,
    prelude::*,
    render::{
        Extract, Render, RenderApp, RenderDebugFlags, RenderStartup, RenderSystems,
        batching::{
            GetBatchData, GetFullBatchData,
            gpu_preprocessing::{
                IndirectParametersCpuMetadata, UntypedPhaseIndirectParametersBuffers,
                batch_and_prepare_sorted_render_phase,
            },
        },
        camera::ExtractedCamera,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{RenderMesh, allocator::MeshAllocator},
        render_asset::RenderAssets,
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_phase::{
            AddRenderCommand, CachedRenderPipelinePhaseItem, DrawFunctionId, DrawFunctions,
            PhaseItem, PhaseItemExtraIndex, SetItemPipeline, SortedPhaseItem,
            SortedRenderPhasePlugin, ViewSortedRenderPhases, sort_phase_system,
        },
        render_resource::{
            BlendState, CachedRenderPipelineId, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Face, FragmentState, PipelineCache, PrimitiveState, RenderPassDescriptor, RenderPipelineDescriptor, SpecializedMeshPipeline, SpecializedMeshPipelineError, SpecializedMeshPipelines, StencilState, StoreOp, TextureFormat, VertexState
        },
        renderer::RenderContext,
        sync_world::MainEntity,
        view::{
            ExtractedView, RenderVisibleEntities, RetainedViewEntity, ViewDepthTexture, ViewTarget,
        },
    },
};

use nonmax::NonMaxU32;

fn main() {
    let asset_root_path = std::env::var("ASSETS_DIR").unwrap_or("assets".into());
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(AssetPlugin {
            file_path: asset_root_path,
            ..Default::default()
        }),
        MeshStencilPhasePlugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, rotate_camera)
    .run();
}

const SHADER_ASSET_PATH: &str = "shaders/custom_stencil.wgsl";

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
        DrawStencil,
    ));

    // another foreground cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(255, 124, 144))),
        Transform::from_translation(Vec3::new(0.6, 0.5, 1.1)),
    ));

    // another background tall cuboid
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 2.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(144, 255, 124))),
        Transform::from_translation(Vec3::new(-0.6, 0.5, -1.1)),
    ));

    // another transparent cuboid
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.5, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba_u8(255, 255, 124, 128),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.6, 0.5, -1.1)),
    ));

    // another outer opaque cuboid
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.2, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(0, 255, 255))),
        Transform::from_translation(Vec3::new(1.2, 0.5, -2.2)),
    ));

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    //camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Off, // TODO: Enable MSAA as a excirse
        SateliteCamera,
    ));
}

#[derive(Component, Debug)]
struct SateliteCamera;

fn rotate_camera(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<SateliteCamera>>,
) {
    for mut transform in &mut query {
        transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(0.3 * time.delta_secs()));
    }
}

#[derive(Component, ExtractComponent, Debug, Clone)]
struct DrawStencil;

struct MeshStencilPhasePlugin;

impl Plugin for MeshStencilPhasePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<DrawStencil>::default(),
            SortedRenderPhasePlugin::<Stencil3d, MeshPipeline>::new(RenderDebugFlags::default()),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedMeshPipelines<StencilPipeline>>()
            .init_resource::<DrawFunctions<Stencil3d>>()
            .add_render_command::<Stencil3d, DrawMesh3dStencil>()
            .init_resource::<ViewSortedRenderPhases<Stencil3d>>()
            .add_systems(RenderStartup, init_stencil_pipeline)
            .add_systems(ExtractSchedule, extract_camera_phases)
            .add_systems(
                Render,
                (
                    queue_custom_meshes.in_set(RenderSystems::QueueMeshes),
                    sort_phase_system::<Stencil3d>.in_set(RenderSystems::PhaseSort),
                    batch_and_prepare_sorted_render_phase::<Stencil3d, StencilPipeline>
                        .in_set(RenderSystems::PrepareResources),
                ),
            );

        render_app
            .add_render_graph_node::<ViewNodeRunner<CustomDrawNode>>(Core3d, CustomDrawPassLabel)
            .add_render_graph_edges(Core3d, (Node3d::MainOpaquePass, CustomDrawPassLabel));
    }
}

#[derive(Resource)]
struct StencilPipeline {
    mesh_pipeline: MeshPipeline,
    shader_handle: Handle<Shader>,
}

fn init_stencil_pipeline(
    mut commands: Commands,
    mesh_pipeline: Res<MeshPipeline>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(StencilPipeline {
        mesh_pipeline: mesh_pipeline.clone(),
        shader_handle: asset_server.load(SHADER_ASSET_PATH),
    });
}

impl SpecializedMeshPipeline for StencilPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &bevy::mesh::MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut vertex_attributes = Vec::new();
        if layout.0.contains(Mesh::ATTRIBUTE_POSITION) {
            vertex_attributes.push(Mesh::ATTRIBUTE_POSITION.at_shader_location(0));
        }

        let vertex_buffer_layout = layout.0.get_layout(&vertex_attributes)?;
        let view_layout = self
            .mesh_pipeline
            .get_view_layout(MeshPipelineViewLayoutKey::from(key));

        Ok(RenderPipelineDescriptor {
            label: Some("Specialized Mesh Pipeline".into()),
            layout: vec![
                view_layout.main_layout.clone(),
                view_layout.empty_layout.clone(),
                self.mesh_pipeline.mesh_layouts.model_only.clone(),
            ],
            vertex: VertexState {
                shader: self.shader_handle.clone(),
                buffers: vec![vertex_buffer_layout],
                ..default()
            },
            fragment: Some(FragmentState {
                shader: self.shader_handle.clone(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            primitive: PrimitiveState {
                topology: key.primitive_topology(),
                cull_mode: Some(Face::Back),
                ..default()
            },
            // Only render if it's overlapped by others
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default()
            }),
            ..default()
        })
    }
}

type DrawMesh3dStencil = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshViewEmptyBindGroup<1>,
    SetMeshBindGroup<2>,
    DrawMesh,
);

struct Stencil3d {
    pub sort_key: FloatOrd,
    pub entity: (Entity, MainEntity),
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub batch_range: Range<u32>,
    pub extra_index: PhaseItemExtraIndex,
    pub indexed: bool,
}

impl PhaseItem for Stencil3d {
    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    #[inline]
    fn entity(&self) -> Entity {
        self.entity.0
    }

    #[inline]
    fn main_entity(&self) -> MainEntity {
        self.entity.1
    }

    #[inline]
    fn batch_range(&self) -> &Range<u32> {
        &self.batch_range
    }

    #[inline]
    fn batch_range_mut(&mut self) -> &mut Range<u32> {
        &mut self.batch_range
    }

    #[inline]
    fn extra_index(&self) -> PhaseItemExtraIndex {
        self.extra_index.clone()
    }

    #[inline]
    fn batch_range_and_extra_index_mut(&mut self) -> (&mut Range<u32>, &mut PhaseItemExtraIndex) {
        (&mut self.batch_range, &mut self.extra_index)
    }
}

impl SortedPhaseItem for Stencil3d {
    type SortKey = FloatOrd;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }

    #[inline]
    fn sort(items: &mut [Self]) {
        items.sort_by_key(|item| item.sort_key());
    }

    #[inline]
    fn indexed(&self) -> bool {
        self.indexed
    }
}

impl CachedRenderPipelinePhaseItem for Stencil3d {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline
    }
}

impl GetBatchData for StencilPipeline {
    type Param = (
        SRes<RenderMeshInstances>,
        SRes<RenderAssets<RenderMesh>>,
        SRes<MeshAllocator>,
    );
    type CompareData = AssetId<Mesh>;
    type BufferData = MeshUniform;

    fn get_batch_data(
        (mesh_instances, _render_assets, mesh_allocator): &SystemParamItem<Self::Param>,
        (_entity, main_entity): (Entity, MainEntity),
    ) -> Option<(Self::BufferData, Option<Self::CompareData>)> {
        let RenderMeshInstances::CpuBuilding(ref mesh_instances) = **mesh_instances else {
            error!("`get_batch_data` should never be called in GPU mesh uniform building mode");
            return None;
        };
        let mesh_instance = mesh_instances.get(&main_entity)?;
        let first_vertex_index =
            match mesh_allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id) {
                Some(mesh_vertex_slice) => mesh_vertex_slice.range.start,
                None => 0,
            };
        let mesh_uniform = {
            let mesh_transforms = &mesh_instance.transforms;
            let (local_from_world_transpose_a, local_from_world_transpose_b) =
                mesh_transforms.world_from_local.inverse_transpose_3x3();
            MeshUniform {
                world_from_local: mesh_transforms.world_from_local.to_transpose(),
                previous_world_from_local: mesh_transforms.previous_world_from_local.to_transpose(),
                lightmap_uv_rect: UVec2::ZERO,
                local_from_world_transpose_a,
                local_from_world_transpose_b,
                flags: mesh_transforms.flags,
                first_vertex_index,
                current_skin_index: u32::MAX,
                material_and_lightmap_bind_group_slot: 0,
                tag: 0,
                pad: 0,
            }
        };
        Some((mesh_uniform, None))
    }
}

impl GetFullBatchData for StencilPipeline {
    type BufferInputData = MeshInputUniform;

    fn get_index_and_compare_data(
        (mesh_instances, _, _): &SystemParamItem<Self::Param>,
        main_entity: MainEntity,
    ) -> Option<(NonMaxU32, Option<Self::CompareData>)> {
        let RenderMeshInstances::GpuBuilding(ref mesh_instances) = **mesh_instances else {
            error!(
                "`get_index_and_compare_data` should never be called in CPU mesh uniform building mode"
            );
            return None;
        };
        let mesh_instance = mesh_instances.get(&main_entity)?;
        Some((
            mesh_instance.current_uniform_index,
            mesh_instance
                .should_batch()
                .then_some(mesh_instance.mesh_asset_id),
        ))
    }

    fn get_binned_batch_data(
        (mesh_instances, _render_assets, mesh_allocator): &SystemParamItem<Self::Param>,
        main_entity: MainEntity,
    ) -> Option<Self::BufferData> {
        let RenderMeshInstances::CpuBuilding(ref mesh_instances) = **mesh_instances else {
            error!(
                "`get_binned_batch_data` should never be called in GPU mesh uniform building mode"
            );
            return None;
        };
        let mesh_instance = mesh_instances.get(&main_entity)?;
        let first_vertex_index =
            match mesh_allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id) {
                Some(mesh_vertex_slice) => mesh_vertex_slice.range.start,
                None => 0,
            };
        Some(MeshUniform::new(
            &mesh_instance.transforms,
            first_vertex_index,
            mesh_instance.material_bindings_index.slot,
            None,
            None,
            None,
        ))
    }

    fn write_batch_indirect_parameters_metadata(
        indexed: bool,
        base_output_index: u32,
        batch_set_index: Option<NonMaxU32>,
        indirect_parameters_buffers: &mut UntypedPhaseIndirectParametersBuffers,
        indirect_parameters_offset: u32,
    ) {
        let indirect_parameters = IndirectParametersCpuMetadata {
            base_output_index,
            batch_set_index: match batch_set_index {
                None => !0,
                Some(batch_set_index) => u32::from(batch_set_index),
            },
        };

        if indexed {
            indirect_parameters_buffers
                .indexed
                .set(indirect_parameters_offset, indirect_parameters);
        } else {
            indirect_parameters_buffers
                .non_indexed
                .set(indirect_parameters_offset, indirect_parameters);
        }
    }

    fn get_binned_index(_: &SystemParamItem<Self::Param>, _: MainEntity) -> Option<NonMaxU32> {
        None
    }
}

fn extract_camera_phases(
    mut stencil_phases: ResMut<ViewSortedRenderPhases<Stencil3d>>,
    cameras: Extract<Query<(Entity, &Camera), With<Camera3d>>>,
    mut live_entities: Local<HashSet<RetainedViewEntity>>,
) {
    live_entities.clear();
    for (main_entity, camera) in &cameras {
        if !camera.is_active {
            continue;
        }
        let retained_view_entity = RetainedViewEntity::new(main_entity.into(), None, 0);
        stencil_phases.insert_or_clear(retained_view_entity);
        live_entities.insert(retained_view_entity);
    }

    stencil_phases.retain(|camera_entity, _| live_entities.contains(camera_entity));
}

#[allow(clippy::too_many_arguments)]
fn queue_custom_meshes(
    custom_draw_functions: Res<DrawFunctions<Stencil3d>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<StencilPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    custom_draw_pipeline: Res<StencilPipeline>,
    render_meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    mut custom_render_phases: ResMut<ViewSortedRenderPhases<Stencil3d>>,
    mut views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    has_marker: Query<(), With<DrawStencil>>,
) {
    for (view, visible_entities, msaa) in &mut views {
        let Some(custom_phase) = custom_render_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };
        let draw_custom = custom_draw_functions.read().id::<DrawMesh3dStencil>();

        let view_key = MeshPipelineKey::from_msaa_samples(msaa.samples())
            | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();

        for (render_entity, visible_entity) in visible_entities.iter::<Mesh3d>() {
            if has_marker.get(*render_entity).is_err() {
                continue;
            }
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(*visible_entity)
            else {
                continue;
            };
            let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };

            let mut mesh_key = view_key;
            mesh_key |= MeshPipelineKey::from_primitive_topology(mesh.primitive_topology());

            let pipeline_id = pipelines.specialize(
                &pipeline_cache,
                &custom_draw_pipeline,
                mesh_key,
                &mesh.layout,
            );
            let pipeline_id = match pipeline_id {
                Ok(id) => id,
                Err(e) => {
                    error!("Failed to specialize pipeline: {e}");
                    continue;
                }
            };
            let distance = rangefinder.distance(&mesh_instance.center);

            custom_phase.add(Stencil3d {
                sort_key: FloatOrd(distance),
                entity: (*render_entity, *visible_entity),
                pipeline: pipeline_id,
                draw_function: draw_custom,
                batch_range: 0..1, // not batched
                extra_index: PhaseItemExtraIndex::None,
                indexed: mesh.indexed(),
            });
        }
    }
}

#[derive(RenderLabel, Debug, Clone, Hash, PartialEq, Eq)]
struct CustomDrawPassLabel;

#[derive(Default)]
struct CustomDrawNode;
impl ViewNode for CustomDrawNode {
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static ExtractedView,
        &'static ViewTarget,
        Option<&'static MainPassResolutionOverride>,
        Read<ViewDepthTexture>,
    );

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (camera, view, target, resolution_override, depth_texture): QueryItem<
            'w,
            '_,
            Self::ViewQuery,
        >,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let Some(stencil_phases) = world.get_resource::<ViewSortedRenderPhases<Stencil3d>>() else {
            return Ok(());
        };

        let view_entity = graph.view_entity();

        let Some(stencil_phase) = stencil_phases.get(&view.retained_view_entity) else {
            return Ok(());
        };

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("stencil pass"),
            color_attachments: &[Some(target.get_color_attachment())],
            depth_stencil_attachment: Some(depth_texture.get_attachment(StoreOp::Store)),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if let Some(viewport) =
            Viewport::from_viewport_and_override(camera.viewport.as_ref(), resolution_override)
        {
            render_pass.set_camera_viewport(&viewport);
        }

        if let Err(err) = stencil_phase.render(&mut render_pass, world, view_entity) {
            error!("Error encountered while rendering the stencil phase {err:?}");
        }

        Ok(())
    }
}
