use bevy::{
    asset::RenderAssetUsages, log::info, math::{
        Dir3,
        primitives::{Measured2d, Primitive3d},
    }, mesh::{Mesh, MeshBuilder, Meshable}
};

#[derive(Copy, Clone)]
pub struct Ring3d {
    pub normal: Dir3,
    pub outer_radius: f32,
    pub inner_radius: f32,

    ///
    /// up end's vertice has UV (0.5, 1.0)
    /// the origin is the oposit of up, which has UV (0.0/1.0, 1.0)
    /// the ring is generated in clockwise direction
    ///
    pub up: Dir3,
    pub resolution: u32,
}

impl Primitive3d for Ring3d {}

impl Ring3d {
    #[inline]
    pub fn new(normal: Dir3, outer_radius: f32, inner_radius: f32) -> Self {
        // up
        // try cross of Dir3::X and normal, if they are parallel, use Dir3::Y instead
        let up = normal
            .cross(Dir3::X.as_vec3())
            .try_normalize()
            .unwrap_or_else(|| normal.cross(Dir3::Y.as_vec3()).normalize())
            .try_into()
            .unwrap();
        info!("Ring3d::new: normal = {:?}, up = {:?}", normal, up);
        let resolution = 32;
        Self {
            normal,
            outer_radius,
            inner_radius,
            up,
            resolution,
        }
    }

    pub fn with_resolution(mut self, resolution: u32) -> Self {
        self.resolution = resolution;
        self
    }
}

impl Measured2d for Ring3d {
    fn area(&self) -> f32 {
        self.outer_radius * self.outer_radius * std::f32::consts::PI
            - self.inner_radius * self.inner_radius * std::f32::consts::PI
    }

    fn perimeter(&self) -> f32 {
        2.0 * std::f32::consts::PI * (self.outer_radius + self.inner_radius)
    }
}

impl Meshable for Ring3d {
    type Output = Ring3dBuilder;
    fn mesh(&self) -> Self::Output {
        Ring3dBuilder {
            normal: self.normal,
            up: self.up,
            outer_radius: self.outer_radius,
            inner_radius: self.inner_radius,
            resolution: self.resolution,
        }
    }
}

pub struct Ring3dBuilder {
    normal: Dir3,
    up: Dir3,
    outer_radius: f32,
    inner_radius: f32,
    resolution: u32,
}

impl MeshBuilder for Ring3dBuilder {
    fn build(&self) -> bevy::mesh::Mesh {
        let start = self.up.normalize();
        let rot90 = self.normal.cross(start).normalize();

        let mut vertices = Vec::new();
        let mut uvs = Vec::new();
        // normals are all the same as self.normal

        // Make a ring with self.resolution segments
        // the last segment is the same as the first one without uvs (u = 1.0)
        for i in 0..=self.resolution {
            let u = i as f32 / self.resolution as f32;
            let angle = u * 2.0 * std::f32::consts::PI;
            let unit = start * angle.cos() + rot90 * angle.sin();
            vertices.push(unit * self.inner_radius);
            vertices.push(unit * self.outer_radius);
            uvs.push([u, 0.0]);
            uvs.push([u, 1.0]);
        }

        let indices: Vec<u32> = (0..((self.resolution+1) * 2)).collect();
        let normals = vec![self.normal.as_vec3(); ((self.resolution+1) * 2) as usize];

        Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleStrip,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(bevy::mesh::Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute( Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}
