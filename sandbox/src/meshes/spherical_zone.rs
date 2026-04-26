use bevy::{
    asset::RenderAssetUsages, log::*, math::{
        Vec3, primitives::{Measured2d, Primitive3d}
    }, mesh::{Mesh, MeshBuilder, Meshable}
};

#[derive(Copy, Clone)]
pub struct SphericalZone {
    radius: f32,
    angle_up: f32,
    angle_down: f32,

    /// resolution for circle.
    circle_resolution: u32,
    /// resolution for angle.
    angle_resolution: u32,
    is_double_sided: bool, // TODO: Add option for only inner side. (currently, it is only for both sides or outer side only.)
}

impl Primitive3d for SphericalZone {}

impl SphericalZone {
    #[inline]
    pub fn new(radius: f32, angle_up: f32, angle_down: f32) -> Self {
        assert!(radius > 0.0, "radius must be positive");
        assert!(angle_up >= 0.0, "angle_up must be non-negative");
        assert!(angle_down > angle_up, "angle_down must be greater than angle_up");
        assert!(angle_down <= std::f32::consts::PI, "angle_down must be less than or equal to PI");
        Self {
            radius,
            angle_up,
            angle_down,
            circle_resolution: 32,
            angle_resolution: 16,
            is_double_sided: false,
        }
    }

    pub fn with_double_sided(mut self, is_double_sided: bool) -> Self {
        self.is_double_sided = is_double_sided;
        self
    }

    pub fn with_circle_resolution(mut self, circle_resolution: u32) -> Self {
        assert!(circle_resolution > 0, "circle_resolution must be positive");
        self.circle_resolution = circle_resolution;
        self
    }

    pub fn with_angle_resolution(mut self, angle_resolution: u32) -> Self {
        assert!(angle_resolution > 0, "angle_resolution must be positive");
        self.angle_resolution = angle_resolution;
        self
    }
}

impl Measured2d for SphericalZone {
    fn area(&self) -> f32 {
        2.0 * std::f32::consts::PI * self.radius * self.radius
            * (self.angle_up.sin() + self.angle_down.sin())
    }

    fn perimeter(&self) -> f32 {
        2.0 * std::f32::consts::PI * self.radius * (self.angle_up.cos() + self.angle_down.cos())
    }
}

impl Meshable for SphericalZone {
    type Output = SphericalZoneMeshBuilder;

    fn mesh(&self) -> Self::Output {
        SphericalZoneMeshBuilder {
            radius: self.radius,
            angle_up: self.angle_up,
            angle_down: self.angle_down,
            circle_resolution: self.circle_resolution,
            angle_resolution: self.angle_resolution,
            is_double_sided: self.is_double_sided,
        }
    }
}



pub struct SphericalZoneMeshBuilder {
    radius: f32,
    angle_up: f32,
    angle_down: f32,
    circle_resolution: u32,
    angle_resolution: u32,
    is_double_sided: bool,
}

impl MeshBuilder for SphericalZoneMeshBuilder {

    fn build(&self) -> Mesh {
        // up: Dir3::Y
        // up edges has v=0 (uv cordinate)
        // start of u (uv cordinate): 0/1 at the Dir3::Z (Forward). it is CCW from the top view.
        
        let num_of_vertices = (self.circle_resolution + 1) * (self.angle_resolution + 1) * if self.is_double_sided { 2 } else { 1 };
        let num_of_indices = self.circle_resolution * self.angle_resolution * 6 * if self.is_double_sided { 2 } else { 1 };


        let mut vertices = Vec::with_capacity(num_of_vertices as usize);
        let mut uvs = Vec::with_capacity(num_of_vertices as usize);
        let mut normals = Vec::with_capacity(num_of_vertices as usize);
        let mut indices = Vec::with_capacity(num_of_indices as usize);

        // outer side
        for i in 0..=self.circle_resolution {
            let u = i as f32 / self.circle_resolution as f32;
            let angle = u * 2.0 * std::f32::consts::PI;

            // X/Z CCW
            let dir = Vec3::new(-angle.sin(), 0.0, -angle.cos());
            for j in 0..=self.angle_resolution {
                let v = j as f32 / self.angle_resolution as f32;
                let angle_from_top = self.angle_up + v * (self.angle_down - self.angle_up);
                let unit = dir * angle_from_top.sin() + Vec3::Y * angle_from_top.cos();
                let vertex = self.radius * unit;
                vertices.push(vertex);
                uvs.push([u, v]);
                normals.push(vertex.normalize());
            }
        }
        if self.is_double_sided {
            // inner side
            // we can copy the vertices and uvs.
            // We need to reverse the normals and the indices.

            // copy
            vertices.extend_from_within(..);
            uvs.extend_from_within(..);

            // normals
            normals.extend(normals.clone().into_iter().map(|n| -n));

            // indices follows
        }
        for i in 0..self.circle_resolution {
            for j in 0..self.angle_resolution {
                let idx0 = i * (self.angle_resolution + 1) + j;
                let idx1 = (i + 1) * (self.angle_resolution + 1) + j;
                let idx2 = (i + 1) * (self.angle_resolution + 1) + j + 1;
                let idx3 = i * (self.angle_resolution + 1) + j + 1;
                indices.push(idx0);
                indices.push(idx2);
                indices.push(idx1);
                indices.push(idx0);
                indices.push(idx3);
                indices.push(idx2);

                if self.is_double_sided {
                    // inner side
                    let idx0 = i * (self.angle_resolution + 1) + j;
                    let idx1 = (i + 1) * (self.angle_resolution + 1) + j;
                    let idx2 = (i + 1) * (self.angle_resolution + 1) + j + 1;
                    let idx3 = i * (self.angle_resolution + 1) + j + 1;
                    indices.push(idx0);
                    indices.push(idx1);
                    indices.push(idx2);
                    indices.push(idx0);
                    indices.push(idx2);
                    indices.push(idx3);
                }
            }
        }

        // debug: num of
        debug!("sphere: v: {}/{}, uvs: {}, normals: {}, indices: {}/{}",
                vertices.len(), num_of_vertices,
                uvs.len(),
                normals.len(),
                indices.len(), num_of_indices);

        Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(bevy::mesh::Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}
