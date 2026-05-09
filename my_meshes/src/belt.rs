use bevy::{
    asset::RenderAssetUsages,
    math::{
        Dir3, Vec3,
        primitives::{Measured2d, Primitive3d},
    },
    mesh::{Indices, Mesh, MeshBuilder, Meshable, PrimitiveTopology},
};

/// A belt shape defined by a start point and cross-sectional direction, and an end point and cross-sectional direction.
/// The mesh is a stripe interpolated between these points using spherical linear interpolation (slerp).
#[derive(Copy, Clone, Debug)]
pub struct Belt {
    pub start_pos: Vec3,
    /// Cross-sectional direction at the start point.
    pub start_dir: Dir3,
    pub end_pos: Vec3,
    /// Cross-sectional direction at the end point.
    pub end_dir: Dir3,
    pub width: f32,
    pub resolution: u32,
}

impl Primitive3d for Belt {}

impl Belt {
    #[inline]
    pub fn new(start_pos: Vec3, start_dir: Dir3, end_pos: Vec3, end_dir: Dir3, width: f32) -> Self {
        Self {
            start_pos,
            start_dir,
            end_pos,
            end_dir,
            width,
            resolution: 32,
        }
    }

    pub fn with_resolution(mut self, resolution: u32) -> Self {
        self.resolution = resolution;
        self
    }
}

impl Measured2d for Belt {
    fn area(&self) -> f32 {
        let length = self.start_pos.length() * self.start_pos.angle_between(self.end_pos);
        length * self.width
    }

    fn perimeter(&self) -> f32 {
        let length = self.start_pos.length() * self.start_pos.angle_between(self.end_pos);
        2.0 * (length + self.width)
    }
}

impl Meshable for Belt {
    type Output = BeltMeshBuilder;

    fn mesh(&self) -> Self::Output {
        BeltMeshBuilder {
            start_pos: self.start_pos,
            start_dir: self.start_dir,
            end_pos: self.end_pos,
            end_dir: self.end_dir,
            width: self.width,
            resolution: self.resolution,
        }
    }
}

/// Mesh builder for a [`Belt`].
pub struct BeltMeshBuilder {
    pub start_pos: Vec3,
    pub start_dir: Dir3,
    pub end_pos: Vec3,
    pub end_dir: Dir3,
    pub width: f32,
    pub resolution: u32,
}

impl MeshBuilder for BeltMeshBuilder {
    fn build(&self) -> Mesh {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        let resolution_f = self.resolution as f32;

        for i in 0..=self.resolution {
            let t = i as f32 / resolution_f;

            // Interpolate position using slerp
            // Vec3::slerp handles interpolation along the arc between the two vectors.
            let p = self.start_pos.slerp(self.end_pos, t);

            // Interpolate the side vector (cross-sectional direction) using slerp
            let side = self.start_dir.slerp(self.end_dir, t);

            // Calculate a tangent direction to the path to determine the width offset.
            // We use a small epsilon to find a nearby point and get the direction.
            let forward = if t < 0.99 {
                let p_next = self.start_pos.slerp(self.end_pos, t + 0.01);
                (p_next - p).normalize_or_zero()
            } else {
                let p_prev = self.start_pos.slerp(self.end_pos, t - 0.01);
                (p - p_prev).normalize_or_zero()
            };

            // The normal is perpendicular to both the side vector and the path direction.
            let n = side.as_vec3().cross(forward).normalize_or_zero();
            let half_width = self.width / 2.0;

            let v_left = p + side * half_width;
            let v_right = p - side * half_width;

            vertices.push(v_left);
            vertices.push(v_right);

            normals.push(n);
            normals.push(n);

            uvs.push([t, 0.0]);
            uvs.push([t, 1.0]);
        }

        // Generate indices for the triangle strip (as a TriangleList for simplicity with double-sided)
        for i in 0..self.resolution {
            let base = i * 2;
            let next = (i + 1) * 2;

            // First side (Outer)
            indices.push(base);
            indices.push(base + 1);
            indices.push(next);

            indices.push(base + 1);
            indices.push(next + 1);
            indices.push(next);
        }

        // Double-sided: duplicate vertices and flip normals/indices
        let vertex_count = vertices.len();
        let index_offset = vertex_count as u32;

        // Clone vertices and UVs for the second side
        let mut other_vertices = vertices.clone();
        let other_uvs = uvs.clone();
        let mut other_normals = Vec::with_capacity(vertex_count);

        for n in &normals {
            other_normals.push(-*n);
        }

        vertices.append(&mut other_vertices);
        uvs.extend(other_uvs);
        normals.append(&mut other_normals);

        // Indices for the second side (Inner) with reversed winding
        for i in 0..self.resolution {
            let base = i * 2 + index_offset;
            let next = (i + 1) * 2 + index_offset;

            indices.push(base);
            indices.push(next);
            indices.push(base + 1);

            indices.push(base + 1);
            indices.push(next);
            indices.push(next + 1);
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_belt_mesh() {
        let belt = Belt::new(
            Vec3::X,
            Dir3::Z,
            Vec3::Y,
            Dir3::Z,
            0.1,
        ).with_resolution(10);
        
        let mesh = belt.mesh().build();
        
        // Resolution 10 means 11 steps.
        // Each step has 2 vertices.
        // Total vertices per side: 11 * 2 = 22.
        // Double-sided: 22 * 2 = 44.
        assert_eq!(mesh.count_vertices(), 44);
        
        // 10 segments.
        // Each segment has 2 triangles per side.
        // Each triangle has 3 indices.
        // Indices per side: 10 * 2 * 3 = 60.
        // Double-sided: 60 * 2 = 120.
        assert_eq!(mesh.indices().unwrap().len(), 120);
    }

    #[test]
    fn test_belt_direction() {
        let belt = Belt::new(
            Vec3::X,
            Dir3::Z, // This should be the cross-sectional direction
            Vec3::Y,
            Dir3::Z,
            0.2,
        ).with_resolution(10);

        let mesh = belt.mesh().build();
        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3().unwrap();
        
        // At t=0 (start), p = X, side should be Z.
        // v_left = X + Z * 0.1 = (1, 0, 0.1)
        // v_right = X - Z * 0.1 = (1, 0, -0.1)
        
        let v0 = Vec3::from_array(positions[0]);
        let v1 = Vec3::from_array(positions[1]);
        
        assert_eq!(v0, Vec3::new(1.0, 0.0, 0.1));
        assert_eq!(v1, Vec3::new(1.0, 0.0, -0.1));
    }
}
