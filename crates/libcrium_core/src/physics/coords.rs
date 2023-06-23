use std::{fmt, ops};

use cgmath::{Point2, Point3, Vector2, Vector3};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::strenum::strenum;

/// A minecrium chunk is a regualr quadriprism, with a bottom edge length of `16` in blocks and
/// a customized height.
pub const CHUNK_WIDTH: usize = 16;

/// 3 kinds of the 3-dimentional axes, includes "x", "y" and "z".
///
/// Also used as a block property that represents the axis along whilst this block is oriented.
///
/// See <https://hub.spigotmc.org/javadocs/spigot/org/bukkit/block/data/Orientable.html>
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[strenum(crate = crate)]
pub enum Axis {
    /// the z axis, from north to south.
    #[default]
    Z = "z",
    /// the x axis, from west to east.
    X = "x",
    /// the y axis, from down to up.
    Y = "y",
}

/// 6 directions parallel to the 3-dimentional axes, includes "south", "north", "east", "west", "up"
/// and "down".
///
/// Also used as a block property that represents the face towards which the block is pointing.
///
/// See <https://hub.spigotmc.org/javadocs/spigot/org/bukkit/block/data/Directional.html>
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[strenum(crate = crate)]
pub enum Direction {
    /// South, the direction parallel to the positive z axis.
    #[default]
    South = "south",
    /// Nouth, the direction parallel to the negative z axis.
    North = "north",
    /// East, the direction parallel to the positive x axis.
    East = "east",
    /// West, the direction parallel to the negative x axis.
    West = "west",
    /// Up, the direction parallel to the positive y axis.
    Up = "up",
    /// Down, the direction parallel to the negative y axis.
    Down = "down",
}

/// A 3-dimentional point that represents the global position of a block.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct BlockPosition {
    /// the x component of the position.
    pub x: i32,
    /// the y component of the position.
    pub y: i32,
    /// the z component of the position.
    pub z: i32,
}

impl BlockPosition {
    /// Returns the block position from the specified components.
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Returns the block position from the chunk section position (`.0`) and the block offset in
    /// the chunk (`.1`).
    ///
    /// Assumes that the components of `offset` are ranges from 0 to [`CHUNK_WIDTH`] (exclusive).
    #[must_use]
    pub const fn from_parts(section: Point3<i32>, offset: Vector3<u8>) -> Self {
        Self {
            x: muladd(section.x, offset.x),
            y: muladd(section.y, offset.y),
            z: muladd(section.z, offset.z),
        }
    }

    /// Returns the block position as the chunk section position (`.0`) and the block offset in the
    /// chunk (`.1`).
    ///
    /// Guarantees that the components of `.1` are ranges from 0 to [`CHUNK_WIDTH`] (exclusive).
    #[must_use]
    pub const fn into_parts(self) -> (Point3<i32>, Vector3<u8>) {
        let (qx, rx) = divrem(self.x);
        let (qy, ry) = divrem(self.y);
        let (qz, rz) = divrem(self.z);

        #[rustfmt::skip]
        #[allow(clippy::needless_return)]
        return (
            Point3 { x: qx, y: qy, z: qz },
            Vector3 { x: rx, y: ry, z: rz },
        );
    }

    /// Returns the chunk position where the block is in.
    #[must_use]
    pub const fn chunk(&self) -> ChunkPosition {
        const W: i32 = CHUNK_WIDTH as i32;
        let mut x = self.x / W;
        if self.x % W < 0 {
            x -= 1;
        }

        let mut z = self.z / W;
        if self.z % W < 0 {
            z -= 1;
        }

        ChunkPosition { x, z }
    }

    /// Returns the *euclidean distance* between the two points.
    ///
    /// The method satifies the commutative law.
    #[must_use]
    pub fn euclidean(&self, other: &Self) -> f64 {
        let delta = *self - *other;
        let euclidean2 = delta.x * delta.x + delta.y * delta.y + delta.z * delta.z;
        f64::sqrt(euclidean2 as f64)
    }

    /// Returns the *squared euclidean distance* between the two points.
    ///
    /// The method satifies the commutative law.
    #[must_use]
    pub fn euclidean2(&self, other: &Self) -> i32 {
        let delta = *self - *other;
        delta.x * delta.x + delta.y * delta.y + delta.z * delta.z
    }

    /// Returns the *manhattan distance* between the two points.
    ///
    /// The method satifies the commutative law.
    #[must_use]
    pub fn manhattan(&self, other: &Self) -> i32 {
        let delta = *self - *other;
        delta.x.abs() + delta.y.abs() + delta.z.abs()
    }
}

impl From<BlockPosition> for Point3<i32> {
    #[inline]
    fn from(value: BlockPosition) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Point3<i32>> for BlockPosition {
    #[inline]
    fn from(value: Point3<i32>) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<BlockPosition> for [i32; 3] {
    #[inline]
    fn from(value: BlockPosition) -> Self {
        [value.x, value.y, value.z]
    }
}

impl From<[i32; 3]> for BlockPosition {
    #[inline]
    fn from([x, y, z]: [i32; 3]) -> Self {
        Self { x, y, z }
    }
}

impl AsRef<[i32; 3]> for BlockPosition {
    #[inline]
    fn as_ref(&self) -> &[i32; 3] {
        // SAFETY: `Self` is marked `#[repr(C)]`.
        unsafe { std::mem::transmute(self) }
    }
}

impl AsMut<[i32; 3]> for BlockPosition {
    fn as_mut(&mut self) -> &mut [i32; 3] {
        // SAFETY: `Self` is marked `#[repr(C)]`.
        unsafe { std::mem::transmute(self) }
    }
}

impl ops::Add<Vector3<i32>> for BlockPosition {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Vector3<i32>) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::Sub<Vector3<i32>> for BlockPosition {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Vector3<i32>) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Sub for BlockPosition {
    type Output = Vector3<i32>;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::AddAssign<Vector3<i32>> for BlockPosition {
    fn add_assign(&mut self, rhs: Vector3<i32>) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::SubAssign<Vector3<i32>> for BlockPosition {
    fn sub_assign(&mut self, rhs: Vector3<i32>) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl ops::Index<Axis> for BlockPosition {
    type Output = i32;

    #[inline]
    fn index(&self, index: Axis) -> &Self::Output {
        match index {
            Axis::Z => &self.z,
            Axis::X => &self.x,
            Axis::Y => &self.y,
        }
    }
}

impl ops::IndexMut<Axis> for BlockPosition {
    #[inline]
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        match index {
            Axis::Z => &mut self.z,
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
        }
    }
}

impl fmt::Debug for BlockPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "block[{}, {}, {}]", self.x, self.y, self.z)
    }
}

impl fmt::Display for BlockPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

impl Serialize for BlockPosition {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        <[i32; 3] as Serialize>::serialize(self.as_ref(), serializer)
    }
}

impl<'de> Deserialize<'de> for BlockPosition {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        <[i32; 3] as Deserialize<'de>>::deserialize(deserializer).map(Self::from)
    }
}

/// A 2-dimentional point that represents the global position of a chunk.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ChunkPosition {
    /// the x component of the position.
    x: i32,
    /// the z component of the position.
    ///
    /// Corresponds to the `y` component of [`Point2`] and [`Vector2`].
    z: i32,
}

impl ChunkPosition {
    /// Returns the chunk position from the specified components.
    #[must_use]
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Returns the *euclidean distance* between the two points.
    ///
    /// The method satifies the commutative law.
    #[must_use]
    pub fn euclidean(&self, other: &Self) -> f64 {
        (self.euclidean2(other) as f64).sqrt()
    }

    /// Returns the *squared euclidean distance* between the two points.
    ///
    /// The method satifies the commutative law.
    #[must_use]
    pub fn euclidean2(&self, other: &Self) -> i32 {
        let delta = *self - *other;
        delta.x * delta.x + delta.y * delta.y
    }

    /// Returns the *manhattan distance* between the two points.
    ///
    #[must_use]
    /// The method satifies the commutative law.
    pub fn manhattan(&self, other: &Self) -> i32 {
        let delta = *self - *other;
        delta.x.abs() + delta.y.abs()
    }
}

impl From<ChunkPosition> for Point2<i32> {
    #[inline]
    fn from(value: ChunkPosition) -> Self {
        Self {
            x: value.x,
            y: value.z,
        }
    }
}

impl From<Point2<i32>> for ChunkPosition {
    #[inline]
    fn from(value: Point2<i32>) -> Self {
        Self {
            x: value.x,
            z: value.y,
        }
    }
}

impl From<ChunkPosition> for [i32; 2] {
    #[inline]
    fn from(value: ChunkPosition) -> Self {
        [value.x, value.z]
    }
}

impl From<[i32; 2]> for ChunkPosition {
    #[inline]
    fn from([x, z]: [i32; 2]) -> Self {
        Self { x, z }
    }
}

impl AsRef<[i32; 2]> for ChunkPosition {
    #[inline]
    fn as_ref(&self) -> &[i32; 2] {
        // SAFETz: `Self` is marked `#[repr(C)]`.
        unsafe { std::mem::transmute(self) }
    }
}

impl AsMut<[i32; 2]> for ChunkPosition {
    fn as_mut(&mut self) -> &mut [i32; 2] {
        // SAFETz: `Self` is marked `#[repr(C)]`.
        unsafe { std::mem::transmute(self) }
    }
}

impl ops::Add<Vector2<i32>> for ChunkPosition {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Vector2<i32>) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            z: self.z + rhs.y,
        }
    }
}

impl ops::AddAssign<Vector2<i32>> for BlockPosition {
    fn add_assign(&mut self, rhs: Vector2<i32>) {
        self.x += rhs.x;
        self.z += rhs.y;
    }
}

impl ops::Sub for ChunkPosition {
    type Output = Vector2<i32>;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Vector2 {
            x: self.x - rhs.x,
            y: self.z - rhs.z,
        }
    }
}

impl ops::Sub<Vector2<i32>> for ChunkPosition {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Vector2<i32>) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            z: self.z - rhs.y,
        }
    }
}

impl ops::SubAssign<Vector2<i32>> for ChunkPosition {
    fn sub_assign(&mut self, rhs: Vector2<i32>) {
        self.x -= rhs.x;
        self.z -= rhs.y;
    }
}

impl ops::Index<Axis> for ChunkPosition {
    type Output = i32;

    #[inline]
    fn index(&self, index: Axis) -> &Self::Output {
        match index {
            Axis::Z => &self.z,
            Axis::X => &self.x,
            Axis::Y => panic!("chunk position cannot be indexed by `y` axis"),
        }
    }
}

impl ops::IndexMut<Axis> for ChunkPosition {
    #[inline]
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        match index {
            Axis::Z => &mut self.z,
            Axis::X => &mut self.x,
            Axis::Y => panic!("chunk position cannot be indexed by `y` axis"),
        }
    }
}

impl fmt::Debug for ChunkPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chunk[{}, {}]", self.x, self.z)
    }
}

impl fmt::Display for ChunkPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.z)
    }
}

impl Serialize for ChunkPosition {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        <[i32; 2] as Serialize>::serialize(self.as_ref(), serializer)
    }
}

impl<'de> Deserialize<'de> for ChunkPosition {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        <[i32; 2] as Deserialize<'de>>::deserialize(deserializer).map(Self::from)
    }
}

#[inline(always)]
const fn muladd(a: i32, b: u8) -> i32 {
    const W: i32 = CHUNK_WIDTH as i32;
    a * W + b as i32
}

#[inline(always)]
const fn divrem(x: i32) -> (i32, u8) {
    const W: i32 = CHUNK_WIDTH as i32;
    let (mut q, mut r) = (x / W, x % W);

    if r < 0 {
        r += W;
        q -= 1;
    }

    (q, r as u8)
}
