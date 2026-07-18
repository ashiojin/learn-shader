use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub fn change_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    background: Res<BackgroundState>,
    q_background: Query<Entity, With<Background>>,
) {
    for entity in q_background.iter() {
        commands.entity(entity).try_despawn();
    }
    match *background {
        BackgroundState::None => {
            // do nothing
        }
        BackgroundState::CheckerboardGround => {
            //spawn_background_checkerboard(&mut commands, &mut meshes, &mut materials, &mut images);
            let color1 = css::WHITE;
            let color2 = css::DARK_GRAY;
            let tile_width = 64;
            let num_checks = 2;
            let mut texture =
                create_checkrboard_texture(tile_width, num_checks, color1.into(), color2.into());
            let mut sampler_desc = ImageSamplerDescriptor::linear();
            sampler_desc.address_mode_u = ImageAddressMode::Repeat;
            sampler_desc.address_mode_v = ImageAddressMode::Repeat;
            texture.sampler = ImageSampler::Descriptor(sampler_desc);
            spawn_background_as_tile(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut images,
                texture,
                GROUND_HEIGHT,
                GROUND_SIZE,
                1.0, // tile size
            );
        }
        BackgroundState::GraphPeperGround => {
            let color1 = css::WHITE;
            let color2 = css::DARK_GRAY; // line
            let tile_width = 64;
            let line_width = 1;
            let mut texture = create_graph_paper_cell_texture(
                tile_width,
                line_width,
                color2.into(),
                color1.into(),
            );
            let mut sampler_desc = ImageSamplerDescriptor::linear();
            sampler_desc.address_mode_u = ImageAddressMode::Repeat;
            sampler_desc.address_mode_v = ImageAddressMode::Repeat;
            texture.sampler = ImageSampler::Descriptor(sampler_desc);
            spawn_background_as_tile(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut images,
                texture,
                GROUND_HEIGHT,
                GROUND_SIZE,
                1.0, // tile size
            );
        }
        BackgroundState::UvGround => {
            let tile_width = 64;
            let texture = create_uv_texture(tile_width);
            spawn_background_as_tile(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut images,
                texture,
                GROUND_HEIGHT,
                GROUND_SIZE,
                1.0, // tile size
            );
        }
    }
}
#[derive(Resource, Debug, Default)]
pub enum BackgroundState {
    None,
    #[default]
    CheckerboardGround,
    GraphPeperGround,
    UvGround,
}

impl BackgroundState {
    pub fn next(&mut self) {
        *self = match self {
            BackgroundState::None => BackgroundState::CheckerboardGround,
            BackgroundState::CheckerboardGround => BackgroundState::GraphPeperGround,
            BackgroundState::GraphPeperGround => BackgroundState::UvGround,
            BackgroundState::UvGround => BackgroundState::None,
        }
    }
}

#[derive(Component, Debug)]
pub struct Background;

fn create_checkrboard_texture(
    size: usize,
    num_checks: usize,
    color1: Color,
    color2: Color,
) -> Image {
    let mut data = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let i_x = x * num_checks / size;
            let i_y = y * num_checks / size;
            let color = if (i_x + i_y).is_multiple_of(2) {
                color1
            } else {
                color2
            };
            data.push((color.to_srgba().red * 255.0) as u8);
            data.push((color.to_srgba().green * 255.0) as u8);
            data.push((color.to_srgba().blue * 255.0) as u8);
            data.push((color.to_srgba().alpha * 255.0) as u8);
        }
    }
    Image::new_fill(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn create_graph_paper_cell_texture(
    size: usize,
    line_width: usize,
    color1: Color,
    color2: Color,
) -> Image {
    // Fill the texture with a graph paper cell pattern.
    // `X` represents the line, and ` ` represents the background.
    // +---X---+
    // |   X   |
    // |   X   |
    // |   X   |
    // XXXXXXXXX
    // |   X   |
    // |   X   |
    // |   X   |
    // +---X---+
    let mut data = Vec::with_capacity(size * size * 4);
    let line_x_left = (size / 2).saturating_sub(line_width / 2);
    let line_x_right = (size / 2).saturating_add(line_width / 2);
    let line_y_top = (size / 2).saturating_sub(line_width / 2);
    let line_y_bottom = (size / 2).saturating_add(line_width / 2);
    for y in 0..size {
        for x in 0..size {
            let color = if (x >= line_x_left && x <= line_x_right)
                || (y >= line_y_top && y <= line_y_bottom)
            {
                color1
            } else {
                color2
            };
            data.push((color.to_srgba().red * 255.0) as u8);
            data.push((color.to_srgba().green * 255.0) as u8);
            data.push((color.to_srgba().blue * 255.0) as u8);
            data.push((color.to_srgba().alpha * 255.0) as u8);
        }
    }
    Image::new_fill(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn create_uv_texture(size: usize) -> Image {
    let mut data = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let u = x as f32 / (size - 1) as f32;
            let v = y as f32 / (size - 1) as f32;
            let color = Color::linear_rgb(u, v, 1.0);
            data.push((color.to_srgba().red * 255.0) as u8);
            data.push((color.to_srgba().green * 255.0) as u8);
            data.push((color.to_srgba().blue * 255.0) as u8);
            data.push((color.to_srgba().alpha * 255.0) as u8);
        }
    }
    Image::new_fill(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

const GROUND_HEIGHT: f32 = -1.1; // make sure the ground is below the sample mesh with size 1.0
const GROUND_SIZE: f32 = 20.0;

#[allow(clippy::too_many_arguments)]
fn spawn_background_as_tile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    texture: Image,
    ground_height: f32,
    ground_size: f32,
    tile_size: f32,
) {
    let material = StandardMaterial {
        base_color_texture: Some(images.add(texture)),
        ..Default::default()
    };
    let tile_mesh = Plane3d::new(Dir3::Y.into(), Vec2::splat(tile_size / 2.0));
    for x in 0..(ground_size / tile_size) as i32 {
        for z in 0..(ground_size / tile_size) as i32 {
            let x_pos = (x as f32 - ground_size / 2.0) * tile_size;
            let z_pos = (z as f32 - ground_size / 2.0) * tile_size;
            commands.spawn((
                Mesh3d(meshes.add(tile_mesh)),
                MeshMaterial3d(materials.add(material.clone())),
                Transform::from_xyz(x_pos, ground_height, z_pos),
                Background,
            ));
        }
    }
}

pub fn handle_background_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut background: ResMut<BackgroundState>,
) {
    // press b to toggle background
    if keys.just_pressed(KeyCode::KeyB) {
        background.next();
    }
}

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BackgroundState>().add_systems(
            Update,
            (
                handle_background_input,
                change_background.run_if(resource_changed::<BackgroundState>),
            ),
        );
    }
}
