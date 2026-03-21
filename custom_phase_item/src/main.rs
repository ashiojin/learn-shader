use bevy::{
    camera::{
        primitives::Aabb,
        visibility::{self, VisibilityClass},
    },
    core_pipeline::core_3d::{CORE_3D_DEPTH_FORMAT, Opaque3d, Opaque3dBatchSetKey, Opaque3dBinKey},
    ecs::{change_detection::Tick, system::lifetimeless::SRes},
    mesh::{VertexBufferLayout, VertexFormat},
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_phase::{
            AddRenderCommand, BinnedRenderPhaseType, DrawFunctions, InputUniformIndex, PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, ViewBinnedRenderPhases
        },
        render_resource::{
            BufferUsages, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, FragmentState, IndexFormat, PipelineCache, RawBufferVec, RenderPipeline, RenderPipelineDescriptor, Specializable, Specializer, SpecializerKey, TextureFormat, Variants, VertexAttribute, VertexState, VertexStepMode
        },
        renderer::{RenderDevice, RenderQueue},
        view::{ExtractedView, RenderVisibleEntities},
    },
};
use bytemuck::{Pod, Zeroable};

fn main() {
    let asset_root_path = std::env::var("ASSETS_DIR").unwrap_or("assets".into());
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(AssetPlugin {
            file_path: asset_root_path,
            ..Default::default()
        }),
        ExtractComponentPlugin::<CustomRenderedEntity>::default(),
    ))
    .add_systems(Startup, setup);

    app.sub_app_mut(RenderApp)
        .init_resource::<CustomPhasePipeline>()
        .add_render_command::<Opaque3d, DrawCustomPhaseItemCommands>()
        .add_systems(
            Render,
            prepare_custom_phase_item_buffers.in_set(RenderSystems::Prepare),
        )
        .add_systems(Render, queue_custom_phase_item.in_set(RenderSystems::Queue));

    app.run();
}

#[derive(Clone, Component, ExtractComponent)]
#[require(VisibilityClass)]
#[component(on_add = visibility::add_visibility_class::<CustomRenderedEntity>)]
struct CustomRenderedEntity;

struct DrawCustomPhaseItem;

impl<P> RenderCommand<P> for DrawCustomPhaseItem
where
    P: PhaseItem,
{
    type Param = SRes<CustomPhaseItemBuffers>;

    type ViewQuery = ();

    type ItemQuery = ();

    fn render<'w>(
        _: &P,
        _: bevy::ecs::query::ROQueryItem<'w, '_, Self::ViewQuery>,
        _: Option<bevy::ecs::query::ROQueryItem<'w, '_, Self::ItemQuery>>,
        custom_phase_item_buffers: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        let custom_phase_item_buffers = custom_phase_item_buffers.into_inner();

        pass.set_vertex_buffer(
            0,
            custom_phase_item_buffers
                .vertices
                .buffer()
                .unwrap()
                .slice(..),
        );

        pass.set_index_buffer(
            custom_phase_item_buffers
                .indices
                .buffer()
                .unwrap()
                .slice(..),
            IndexFormat::Uint32,
        );

        pass.draw_indexed(0..3, 0, 0..1);

        RenderCommandResult::Success
    }
}

#[derive(Resource)]
struct CustomPhaseItemBuffers {
    vertices: RawBufferVec<Vertex>,
    indices: RawBufferVec<u32>,
}
impl FromWorld for CustomPhaseItemBuffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        let mut vbo = RawBufferVec::new(BufferUsages::VERTEX);
        let mut ibo = RawBufferVec::new(BufferUsages::INDEX);

        for vertex in &VERTICES {
            vbo.push(*vertex);
        }
        for index in 0..3 {
            ibo.push(index);
        }

        vbo.write_buffer(render_device, render_queue);
        ibo.write_buffer(render_device, render_queue);

        CustomPhaseItemBuffers {
            vertices: vbo,
            indices: ibo,
        }
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: Vec3,
    pad0: u32,
    color: Vec3,
    pad1: u32,
}

impl Vertex {
    const fn new(position: Vec3, color: Vec3) -> Self {
        Self {
            position,
            pad0: 0,
            color,
            pad1: 0,
        }
    }
}

static VERTICES: [Vertex; 3] = [
    Vertex::new(vec3(-0.866, -0.5, 0.5), vec3(1.0, 0.0, 0.0)),
    Vertex::new(vec3(0.866, -0.5, 0.5), vec3(0.0, 1.0, 0.0)),
    Vertex::new(vec3(0.0, 1.0, 0.5), vec3(0.0, 0.0, 1.0)),
];

type DrawCustomPhaseItemCommands = (SetItemPipeline, DrawCustomPhaseItem);

struct CustomPhaseSpecializer;

#[derive(Copy, Clone, PartialEq, Eq, Hash, SpecializerKey)]
struct CustomPhaseKey(Msaa);

impl Specializer<RenderPipeline> for CustomPhaseSpecializer {
    type Key = CustomPhaseKey;

    fn specialize(
        &self,
        key: Self::Key,
        descriptor: &mut <RenderPipeline as Specializable>::Descriptor,
    ) -> Result<bevy::render::render_resource::Canonical<Self::Key>, BevyError> {
        descriptor.multisample.count = key.0.samples();
        Ok(key)
    }
}

#[derive(Resource)]
struct CustomPhasePipeline {
    variants: Variants<RenderPipeline, CustomPhaseSpecializer>,
}

impl FromWorld for CustomPhasePipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/custom_phase_item.wgsl");

        let base_descriptor = RenderPipelineDescriptor {
            label: Some("custom render pipeline".into()),
            vertex: VertexState {
                shader: shader.clone(),
                buffers: vec![VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: vec![
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 16,
                            shader_location: 1,
                        },
                    ],
                }],
                ..Default::default()
            },
            fragment: Some(FragmentState {
                shader: shader.clone(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..Default::default()
            }),
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: default(),
                bias: default(),
            }),
            ..Default::default()
        };

        let variants = Variants::new(CustomPhaseSpecializer, base_descriptor);

        Self { variants }
    }
}

fn prepare_custom_phase_item_buffers(mut commands: Commands) {
    commands.init_resource::<CustomPhaseItemBuffers>();
}

fn queue_custom_phase_item(
    pipeline_cache: Res<PipelineCache>,
    mut pipeline: ResMut<CustomPhasePipeline>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    mut next_tick: Local<Tick>,
) {
    let draw_custom_phase_item = opaque_draw_functions
        .read()
        .id::<DrawCustomPhaseItemCommands>();

    for (view, view_visible_entities, msaa) in views.iter() {
        let Some(opaque_phase) = opaque_render_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };

        for &entity in view_visible_entities.get::<CustomRenderedEntity>().iter() {
            let Ok(pipeline_id) = pipeline
                .variants
                .specialize(&pipeline_cache, CustomPhaseKey(*msaa))
            else {
                continue;
            };

            let this_tick = next_tick.get() + 1;
            next_tick.set(this_tick);

            opaque_phase.add(
                Opaque3dBatchSetKey {
                    draw_function: draw_custom_phase_item,
                    pipeline: pipeline_id,
                    material_bind_group_index: None,
                    lightmap_slab: None,
                    vertex_slab: default(),
                    index_slab: None,
                },
                Opaque3dBinKey {
                    asset_id: AssetId::<Mesh>::invalid().untyped(),
                },
                entity,
                InputUniformIndex::default(),
                BinnedRenderPhaseType::NonMesh,
                *next_tick,
            );
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Visibility::default(),
        Transform::default(),
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::splat(0.5),
        },
        CustomRenderedEntity,
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
