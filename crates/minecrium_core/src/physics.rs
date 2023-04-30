use bevy::math::Vec3A;
use bevy::prelude::ReflectDefault;
use bevy::reflect::{FromReflect, Reflect};
use serde::{Deserialize, Serialize};

/// Determines whether the two primitives intersect.
///
/// # Implementors
///
/// - [`Aabb`] axis-aligned bounding box
/// - [`Sphere`] bouding shpere
pub trait Intersects<Rhs = Self> {
    /// Returns `true` if the object intersects with another object.
    fn intersects(self, rhs: Rhs) -> bool;
}

/// An `Axis-Aligned Bounding Box`.
///
/// # Reference
///
/// - [`Aabb`](bevy::render::primitives::Aabb) in `bevy`.
///
/// - <https://developer.mozilla.org/en-US/docs/Games/Techniques/3D_collision_detection>
#[derive(Clone, Copy, Debug, Default, Reflect, FromReflect, Serialize, Deserialize)]
#[reflect(Default)]
pub struct Aabb {
    /// the minimal position of the bounding box.
    pub min: Vec3A,
    /// the maximal position of the bounding box.
    pub max: Vec3A,
}

impl Aabb {
    /// Returns the extents of the AABB.
    #[inline]
    pub fn extents(&self) -> Vec3A {
        self.max - self.min
    }

    /// Calculate the relative radius of the AABB with respect to a plane
    #[inline]
    pub fn relative_radius(&self, p_normal: &Vec3A, axes: &[Vec3A]) -> f32 {
        // NOTE: dot products on Vec3A use SIMD and even with the overhead of conversion are net faster than Vec3
        let half_extents = (self.max - self.min) / 2.;

        Vec3A::new(
            p_normal.dot(axes[0]),
            p_normal.dot(axes[1]),
            p_normal.dot(axes[2]),
        )
        .abs()
        .dot(half_extents)
    }
}

impl From<Sphere> for Aabb {
    #[inline]
    fn from(value: Sphere) -> Self {
        Self {
            min: value.center - value.radius,
            max: value.center + value.radius,
        }
    }
}

impl From<bevy::render::primitives::Aabb> for Aabb {
    #[inline]
    fn from(value: bevy::render::primitives::Aabb) -> Self {
        Self {
            min: value.min(),
            max: value.max(),
        }
    }
}

impl From<Aabb> for bevy::render::primitives::Aabb {
    #[inline]
    fn from(value: Aabb) -> Self {
        Self {
            center: 0.5 * (value.max + value.min),
            half_extents: 0.5 * (value.max - value.min),
        }
    }
}

impl Intersects<Vec3A> for Aabb {
    #[inline]
    fn intersects(self, rhs: Vec3A) -> bool {
        (self.min.cmplt(rhs) & rhs.cmplt(self.max)).all()
    }
}

impl Intersects<Aabb> for Aabb {
    #[inline]
    fn intersects(self, rhs: Self) -> bool {
        (self.min.cmplt(rhs.max) & rhs.min.cmplt(self.max)).all()
    }
}

impl Intersects<Sphere> for Aabb {
    #[inline]
    fn intersects(self, rhs: Sphere) -> bool {
        rhs.center
            .distance_squared(rhs.center.clamp(self.min, self.max))
            < (rhs.radius * rhs.radius)
    }
}

/// A `Bounding Sphere`.
///
/// # Reference
///
/// - [`Aabb`](bevy::render::primitives::Sphere) in `bevy`.
///
/// - <https://developer.mozilla.org/en-US/docs/Games/Techniques/3D_collision_detection>
#[derive(Clone, Copy, Debug, Default, Reflect, FromReflect, Serialize, Deserialize)]
#[reflect(Default)]
pub struct Sphere {
    /// the center position of the sphere
    pub center: Vec3A,
    /// the raidus of the sphere
    pub radius: f32,
}

impl From<bevy::render::primitives::Sphere> for Sphere {
    #[inline]
    fn from(value: bevy::render::primitives::Sphere) -> Self {
        Self {
            center: value.center,
            radius: value.radius,
        }
    }
}

impl From<Sphere> for bevy::render::primitives::Sphere {
    #[inline]
    fn from(value: Sphere) -> Self {
        Self {
            center: value.center,
            radius: value.radius,
        }
    }
}

impl Intersects<Vec3A> for Sphere {
    #[inline]
    fn intersects(self, rhs: Vec3A) -> bool {
        self.center.distance_squared(rhs) < (self.radius * self.radius)
    }
}

impl Intersects<Aabb> for Sphere {
    #[inline]
    fn intersects(self, rhs: Aabb) -> bool {
        <Aabb as Intersects<Sphere>>::intersects(rhs, self)
    }
}

impl Intersects<Sphere> for Sphere {
    #[inline]
    fn intersects(self, rhs: Sphere) -> bool {
        let distance = self.radius + rhs.radius;
        self.center.distance_squared(rhs.center) < distance * distance
    }
}
