use bevy::{asset::RenderAssetUsages, color::palettes::css, prelude::*, render::render_resource::{Extent3d, TextureDimension, TextureFormat}};


pub fn update_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    background: Res<BackgroundState>,
    q_background: Query<Entity, With<Background>>,
) {
    for entity in q_background.iter() {
        commands.entity(entity).despawn();
    }
    // background.spawn(
    //     &mut commands,
    //     &mut meshes,
    //     &mut materials,
    //     &mut images,
    // );
    match *background {
        BackgroundState::None => {
            // do nothing
        }
        BackgroundState::CheckerboardGround => {
            spawn_background_checkerboard(&mut commands, &mut meshes, &mut materials, &mut images);
        }
    }
}
#[derive(Resource, Debug, Default)]
pub enum BackgroundState {
    None,
    #[default]
    CheckerboardGround,
}

impl BackgroundState {
    pub fn next(&mut self) {
        *self = match self {
            BackgroundState::None => BackgroundState::CheckerboardGround,
            BackgroundState::CheckerboardGround => BackgroundState::None,
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
            let check_x = x * num_checks / size;
            let check_y = y * num_checks / size;
            let color = if (check_x + check_y).is_multiple_of(2) {
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

fn spawn_background_checkerboard(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) {
    let color1 = css::WHITE;
    let color2 = css::BLACK;

    // (-1, -1) ~ (1, 1) is for mesh with size 1.0

    let ground_height = -1.1; // make sure the ground is below the sample mesh with size 1.0
    let ground_size = 20.0;

    let texture = create_checkrboard_texture(512, 16, color1.into(), color2.into());
    let material = StandardMaterial {
        base_color_texture: Some(images.add(texture)),
        ..Default::default()
    };
    let mesh = Plane3d::new(Dir3::Y.into(), Vec2::splat(ground_size));
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        Transform::from_xyz(0., ground_height, 0.),
        Background,
    ));
}
