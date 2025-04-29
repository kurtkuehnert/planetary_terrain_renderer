use crate::math::spheroid::project_point_spheroid;
use bevy::{math::DVec3, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TerrainShape {
    Plane { side_length: f64 },
    Sphere { radius: f64 },
    Spheroid { major_axis: f64, minor_axis: f64 },
}

impl TerrainShape {
    pub const WGS84: Self = TerrainShape::Spheroid {
        major_axis: 6378137.0,
        minor_axis: 6356752.314245,
    };

    pub fn face_size(self) -> f64 {
        2.0 * std::f64::consts::PI / 4.0 * self.scale_scalar()
    }

    pub fn scale_scalar(self) -> f64 {
        match self {
            TerrainShape::Plane { side_length } => side_length / 2.0,
            TerrainShape::Sphere { radius } => radius,
            TerrainShape::Spheroid { major_axis, .. } => major_axis,
        }
    }
    pub fn scale(self) -> DVec3 {
        match self {
            TerrainShape::Plane { side_length } => DVec3::new(side_length, 1.0, side_length),
            TerrainShape::Sphere { radius } => DVec3::splat(radius),
            TerrainShape::Spheroid {
                major_axis,
                minor_axis,
            } => DVec3::new(major_axis, minor_axis, major_axis),
        }
    }

    pub fn transform(self) -> Transform {
        Transform::from_scale(self.scale().as_vec3())
    }
    pub(crate) fn is_spherical(self) -> bool {
        match self {
            TerrainShape::Plane { .. } => false,
            TerrainShape::Sphere { .. } => true,
            TerrainShape::Spheroid { .. } => true,
        }
    }
    pub fn face_count(self) -> u32 {
        if self.is_spherical() { 6 } else { 1 }
    }

    pub fn position_unit_to_local(self, unit_position: DVec3, height: f64) -> DVec3 {
        let local_position = self.scale() * unit_position;
        let local_normal = (self.scale()
            * if self.is_spherical() {
                unit_position
            } else {
                DVec3::Y
            })
        .normalize();

        local_position + height * local_normal
    }

    pub fn position_local_to_unit(self, local_position: DVec3) -> DVec3 {
        match self {
            TerrainShape::Plane { .. } => DVec3::new(1.0, 0.0, 1.0) * local_position / self.scale(),
            TerrainShape::Sphere { .. } => (local_position / self.scale()).normalize(),
            TerrainShape::Spheroid {
                major_axis,
                minor_axis,
            } => {
                let surface_position =
                    project_point_spheroid(major_axis, minor_axis, local_position);
                (surface_position / self.scale()).normalize()
            }
        }
    }
}
