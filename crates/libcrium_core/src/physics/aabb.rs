use std::ops;

use cgmath::prelude::*;
use cgmath::{Point3, Vector3};
use serde::{Deserialize, Serialize};

/// An `Axis-Aligned Bounding Box`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    /// Position of the point with the smallest coordinates.
    pub min: Point3<f32>,
    /// Position of the Point3int with the highest coordinates.
    ///
    /// Each component of `self.min` must be smaller than the related components of `self.max`.
    pub max: Point3<f32>,
}

impl Aabb {
    /// The vertex indices of each edge of this Aabb.
    ///
    /// This gives, for each edge of this Aabb, the indices of its vertices when taken from the
    /// `self.vertices()` array.
    ///
    /// Here is how the faces are numbered, assuming a right-handed coordinate system:
    ///
    /// ```text
    ///    y             3 -- 2
    ///    |           7 −- 6 |
    ///    ___ x       |    | 1  (the zero is bellow 3 and on the left of 1, hidden by the 4-5-6-7 face.)
    ///   /            4 -- 5
    ///  z
    /// ```
    pub const EDGES_VERTEX_INDEXES: [(usize, usize); 12] = [
        (0, 1),
        (1, 2),
        (3, 2),
        (0, 3),
        (4, 5),
        (5, 6),
        (7, 6),
        (4, 7),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    /// The vertex indices of each face of this Aabb.
    ///
    /// This gives, for each face of this Aabb, the indices of its vertices when taken from the
    /// `self.vertices()` array.
    ///
    /// Here is how the faces are numbered, assuming a right-handed coordinate system:
    ///
    /// ```text
    ///    y             3 -- 2
    ///    |           7 −- 6 |
    ///    ___ x       |    | 1  (the zero is bellow 3 and on the left of 1, hidden by the 4-5-6-7 face.)
    ///   /            4 -- 5
    ///  z
    /// ```
    pub const FACES_VERTEX_INDEXES: [(usize, usize, usize, usize); 6] = [
        (1, 2, 6, 5),
        (0, 3, 7, 4),
        (2, 3, 7, 6),
        (1, 0, 4, 5),
        (4, 5, 6, 7),
        (0, 1, 2, 3),
    ];

    /// Returns the [`Aabb`] from the given two points.
    #[must_use]
    pub fn from_points(one: Point3<f32>, two: Point3<f32>) -> Self {
        let min = Point3::zip(one, two, f32::min);
        let max = Point3::zip(one, two, f32::max);
        Self { min, max }
    }

    /// Returns the [`Aabb`] from the given min position and max position.
    ///
    /// # Safety
    ///
    /// Each component of `min` must be smaller than or equal to the corresponding components of
    /// `max`.
    #[inline]
    #[must_use]
    pub fn from_min_max(min: Point3<f32>, max: Point3<f32>) -> Self {
        debug_assert!(min.x <= max.x && min.y <= max.y && min.z <= max.z);

        Self { min, max }
    }

    /// Returns the [`Aabb`] from the given center and half-extents.
    ///
    /// # Safety
    ///
    /// Each component of `half_extents` must be greater than or equal to `0.0`.
    #[inline]
    #[must_use]
    pub fn from_half_extents(center: Point3<f32>, half_extents: Vector3<f32>) -> Self {
        debug_assert!(half_extents.x >= 0.0 && half_extents.y >= 0.0 && half_extents.z >= 0.0);

        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Returns the center point of this aabb.
    #[inline]
    #[must_use]
    pub fn center(&self) -> Point3<f32> {
        Point3::midpoint(self.min, self.max)
    }

    /// Returns the extents of this aabb.
    #[inline]
    #[must_use]
    pub fn extents(&self) -> Vector3<f32> {
        self.max - self.min
    }

    /// Returns the half extents of this aabb.
    #[inline]
    #[must_use]
    pub fn half_extents(&self) -> Vector3<f32> {
        self.extents() / 2.0
    }

    /// Returns the volume of this aabb.
    #[inline]
    #[must_use]
    pub fn volume(&self) -> f32 {
        self.extents().product()
    }

    /// Returns the vertices of this aabb.
    #[inline]
    #[must_use]
    pub fn vertices(&self) -> [Point3<f32>; 8] {
        [
            Point3::new(self.min.x, self.min.y, self.min.z),
            Point3::new(self.max.x, self.min.y, self.min.z),
            Point3::new(self.max.x, self.max.y, self.min.z),
            Point3::new(self.min.x, self.max.y, self.min.z),
            Point3::new(self.min.x, self.min.y, self.max.z),
            Point3::new(self.max.x, self.min.y, self.max.z),
            Point3::new(self.max.x, self.max.y, self.max.z),
            Point3::new(self.min.x, self.max.y, self.max.z),
        ]
    }

    /// Returns the scaled aabb of this aabb.
    #[inline]
    #[must_use]
    pub fn scale(self, scale: Vector3<f32>) -> Self {
        let scale = scale.map(f32::abs);

        Self {
            min: Point3 {
                x: self.min.x * scale.x,
                y: self.min.y * scale.y,
                z: self.min.z * scale.z,
            },
            max: Point3 {
                x: self.max.x * scale.x,
                y: self.max.y * scale.y,
                z: self.max.z * scale.z,
            },
        }
    }

    /// Returns the lossend aabb of this aabb.
    ///
    /// Requires that `amount >= 0.0`.
    #[inline]
    #[must_use]
    pub fn loosen(&self, amount: f32) -> Self {
        assert!(0.0 <= amount, "The loosening margin must be positive.");
        let amount = Vector3::from([amount; 3]);

        Self {
            min: self.min - amount,
            max: self.max + amount,
        }
    }

    /// Returns the lossend tightened of this aabb.
    ///
    /// Requires that `amount >= 0.0`.
    #[inline]
    #[must_use]
    pub fn tighten(&self, amount: f32) -> Self {
        assert!(0.0 <= amount, "The tightening margin must be positive.");
        let amount = Vector3::from([amount; 3]);

        Self {
            min: self.min + amount,
            max: self.max - amount,
        }
    }

    /// Returns the merged aabb of the two aabbs.
    ///
    /// Equivalent to [`self + other`](ops::Add).
    #[inline]
    #[must_use]
    pub fn merge(&self, other: Self) -> Self {
        Self {
            min: Point3::zip(self.min, other.min, f32::min),
            max: Point3::zip(self.max, other.max, f32::max),
        }
    }

    /// Returns `true` if this aabb contains `other`.
    #[inline]
    #[must_use]
    pub fn contains(&self, other: Self) -> bool {
        // self.min <= other.min && other.max <= self.max
        self.min.x <= other.min.x
            && self.min.y <= other.min.y
            && self.min.z <= other.min.z
            && other.max.x <= self.max.x
            && other.max.y <= self.max.y
            && other.max.z <= self.max.z
    }

    /// Computes the intersection of this Aabb and another one.
    ///
    /// Equivalent to [`self - other`](ops::Add).
    #[must_use]
    pub fn intersection(&self, other: Self) -> Option<Aabb> {
        let min = Point3::zip(self.min, other.min, f32::max);
        let max = Point3::zip(self.max, other.max, f32::min);

        if min.x < max.x || min.y < max.y || min.z < max.z {
            Some(Self { min, max })
        } else {
            None
        }
    }

    /// Returns the difference between this aabb and `other`.
    ///
    /// Removing another aabb from `self` will result in up to 6 new smaller Aabbs.
    #[must_use]
    pub fn difference(&self, other: Self) -> Vec<Aabb> {
        if self.intersection(other).is_none() {
            // special case when the two boxes are disjoint.
            Vec::from([*self])
        } else {
            // common case when the box intersects with `other`.
            let mut result = Vec::with_capacity(6);
            let mut rest = *self;

            for i in 0..3 {
                if other.min[i] > rest.min[i] {
                    let mut fragment = rest;
                    fragment.max[i] = other.min[i];
                    rest.min[i] = other.min[i];
                    result.push(fragment);
                }

                if other.max[i] < rest.max[i] {
                    let mut fragment = rest;
                    fragment.min[i] = other.max[i];
                    rest.max[i] = other.max[i];
                    result.push(fragment);
                }
            }

            result
        }
    }

    /// Splits this box at its center, into height parts (as in an octree).
    #[must_use]
    pub fn split_at_center(&self) -> [Aabb; 8] {
        let center = self.center();
        [
            Self {
                min: Point3::new(self.min.x, self.min.y, self.min.z),
                max: Point3::new(center.x, center.y, center.z),
            },
            Self {
                min: Point3::new(center.x, self.min.y, self.min.z),
                max: Point3::new(self.max.x, center.y, center.z),
            },
            Self {
                min: Point3::new(center.x, center.y, self.min.z),
                max: Point3::new(self.max.x, self.max.y, center.z),
            },
            Self {
                min: Point3::new(self.min.x, center.y, self.min.z),
                max: Point3::new(center.x, self.max.y, center.z),
            },
            Self {
                min: Point3::new(self.min.x, self.min.y, center.z),
                max: Point3::new(center.x, center.y, self.max.z),
            },
            Self {
                min: Point3::new(center.x, self.min.y, center.z),
                max: Point3::new(self.max.x, center.y, self.max.z),
            },
            Self {
                min: Point3::new(center.x, center.y, center.z),
                max: Point3::new(self.max.x, self.max.y, self.max.z),
            },
            Self {
                min: Point3::new(self.min.x, center.y, center.z),
                max: Point3::new(center.x, self.max.y, self.max.z),
            },
        ]
    }
}

impl ops::Add for Aabb {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.merge(rhs)
    }
}

impl ops::Sub for Aabb {
    type Output = Vec<Self>;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.difference(rhs)
    }
}
