//! Defines the minecrium coordinate system.
//!
//! The minecrium uses the **right-handed coordinate system**, the relationship between direction
//! and axis is demonstrated in the following graph.
//!
//! ```text
//!               up(Y)
//!                 |   north
//!                 |  /
//!                 | /
//!                 |/
//! west <----------+----------> east(X)
//!                /|
//!               / |
//!              /  |
//!        south(Z) |
//!                 down
//! ```
//!
//! A minecrium chunk is a regualr quadriprism, with an bottom edge length of `16` (in blocks) and
//! a customized height.
//!
//! # Overview
//!
//! | items                    | description                                                       |
//! | ------------------------ | ----------------------------------------------------------------- |
//! | [`Axis`]                 | 3 kinds of the 3-dimentional axes.                                |
//! | [`HAxis`]                | 2 kinds of horizontal axes.                                       |
//! | [`Direction`]            | 6 directions parallel to the 3-dimentional axes.                  |
//! | [`HDirection`]           | 8 kinds of horizontal directions.                                 |
//! | [`ChunkPosition`]        | Absolute position of a chunk.                                     |
//! | [`BlockPosition`]        | Absolute position of a block.                                     |
//! | [`BlockOffset`]          | Relative position of a block in the chunk.                        |

use std::str::FromStr;
use std::{fmt, ops};

use cgmath::Vector3;
use serde::{Deserialize, Serialize};

use crate::errors::*;

/// The width of a chunk, in blocks (= `16`).
pub const CHUNK_WIDTH: usize = 16;

/// 3 kinds of the 3-dimentional axes, includes "x", "y" and "z".
///
/// See the [`module documentation`](crate::coords) for more details.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    /// the z axis, south-north direction.
    #[default]
    Z,
    /// the x axis, east-west direction.
    X,
    /// the y axis, up-down direction.
    Y,
}

impl AsRef<str> for Axis {
    fn as_ref(&self) -> &str {
        match self {
            Self::Z => "z",
            Self::X => "x",
            Self::Y => "y",
        }
    }
}

impl fmt::Debug for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(<Self as AsRef<str>>::as_ref(self))
    }
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(<Self as AsRef<str>>::as_ref(self))
    }
}

impl FromStr for Axis {
    type Err = ParseAxisError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "z" => Ok(Self::Z),
            "x" => Ok(Self::X),
            "y" => Ok(Self::Y),
            _ => Err(ParseAxisError),
        }
    }
}

/// 2 kinds of horizontal axes, includes "x" and "z".
///
/// See the [`module documentation`](crate::coords) for more details.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HAxis {
    /// the z axis, south-north direction.
    #[default]
    Z,
    /// the x axis, east-west direction.
    X,
}

impl AsRef<str> for HAxis {
    fn as_ref(&self) -> &str {
        match self {
            Self::Z => "z",
            Self::X => "x",
        }
    }
}

impl fmt::Debug for HAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(<Self as AsRef<str>>::as_ref(self))
    }
}

impl fmt::Display for HAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(<Self as AsRef<str>>::as_ref(self))
    }
}

impl FromStr for HAxis {
    type Err = ParseHAxisError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "z" => Ok(Self::Z),
            "x" => Ok(Self::X),
            _ => Err(ParseHAxisError),
        }
    }
}

/// 6 directions parallel to the 3-dimentional axes, includes "south", "north", "east", "west", "up"
/// and "down".
///
/// See the [`module documentation`](crate::coords) for more details.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// South, the direction parallel to the positive z axis.
    ///
    /// Serialize to `"south"`.
    #[default]
    South,
    /// Nouth, the direction parallel to the negative z axis.
    ///
    /// Serialize to `"north"`.
    North,
    /// East, the direction parallel to the positive x axis.
    ///
    /// Serialize to `"east"`.
    East,
    /// West, the direction parallel to the negative x axis.
    ///
    /// Serialize to `"west"`.
    West,
    /// Up, the direction parallel to the positive y axis.
    ///
    /// Serialize to `"up"`.
    Up,
    /// Down, the direction parallel to the negative y axis.
    ///
    /// Serialize to `"down"`.
    Down,
}

impl Direction {
    /// Returns the direction that is opposite to the given direction.
    pub fn opposite(&self) -> Self {
        match self {
            Self::South => Self::North,
            Self::North => Self::South,
            Self::East => Self::West,
            Self::West => Self::East,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }

    /// Returns the axis which is parallel to the given direction.
    pub fn axis(&self) -> Axis {
        match self {
            Self::South => Axis::Z,
            Self::North => Axis::Z,
            Self::East => Axis::X,
            Self::West => Axis::X,
            Self::Up => Axis::Y,
            Self::Down => Axis::Y,
        }
    }
}

impl AsRef<str> for Direction {
    fn as_ref(&self) -> &str {
        match self {
            Self::South => "south",
            Self::North => "north",
            Self::East => "east",
            Self::West => "west",
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

impl fmt::Debug for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(<Self as AsRef<str>>::as_ref(self))
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(<Self as AsRef<str>>::as_ref(self))
    }
}

impl FromStr for Direction {
    type Err = ParseDirectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "south" => Ok(Self::South),
            "north" => Ok(Self::North),
            "east" => Ok(Self::East),
            "west" => Ok(Self::West),
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            _ => Err(ParseDirectionError),
        }
    }
}

impl From<Direction> for Vector3<i32> {
    fn from(value: Direction) -> Self {
        match value {
            Direction::South => Vector3::new(0, 0, 1),
            Direction::North => Vector3::new(0, 0, -1),
            Direction::East => Vector3::new(1, 0, 0),
            Direction::West => Vector3::new(-1, 0, 0),
            Direction::Up => Vector3::new(0, 1, 0),
            Direction::Down => Vector3::new(0, -1, 0),
        }
    }
}

impl From<Direction> for Vector3<f32> {
    fn from(value: Direction) -> Self {
        match value {
            Direction::South => Vector3::new(0., 0., 1.),
            Direction::North => Vector3::new(0., 0., -1.),
            Direction::East => Vector3::new(1., 0., 0.),
            Direction::West => Vector3::new(-1., 0., 0.),
            Direction::Up => Vector3::new(0., 1., 0.),
            Direction::Down => Vector3::new(0., -1., 0.),
        }
    }
}

/// 8 kinds of horizontal directions, includes "south", "north", "east", "west", "southeast",
/// "southwest", "northeast", "northwest".
///
/// See the [`module documentation`](crate::coords) for more details.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HDirection {
    /// South, the direction parallel to the positive z axis.
    ///
    /// Serialize to `"south"`.
    #[default]
    South,
    /// Nouth, the direction parallel to the negative z axis.
    ///
    /// Serialize to `"north"`.
    North,
    /// East, the direction parallel to the positive x axis.
    ///
    /// Serialize to `"east"`.
    East,
    /// West, the direction parallel to the negative x axis.
    ///
    /// Serialize to `"west"`.
    West,
    /// Southeast, the direction between south and east.
    ///
    /// Serialize to `"southeast"`.
    Southeast,
    /// Southwest, the direction between south and west.
    ///
    /// Serialize to `"southwest"`.
    Southwest,
    /// Northeast, the direction between north and east.
    ///
    /// Serialize to `"northeast"`.
    Northeast,
    /// Northwest, the direction between north and west.
    ///
    /// Serialize to `"northwest"`.
    Northwest,
}

impl AsRef<str> for HDirection {
    fn as_ref(&self) -> &str {
        match self {
            Self::South => "south",
            Self::North => "north",
            Self::East => "east",
            Self::West => "west",
            Self::Southeast => "southeast",
            Self::Southwest => "southwest",
            Self::Northeast => "northeast",
            Self::Northwest => "northwest",
        }
    }
}

impl fmt::Debug for HDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(<Self as AsRef<str>>::as_ref(self))
    }
}

impl fmt::Display for HDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(<Self as AsRef<str>>::as_ref(self))
    }
}

impl FromStr for HDirection {
    type Err = ParseHDirectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "south" => Ok(Self::South),
            "north" => Ok(Self::North),
            "east" => Ok(Self::East),
            "west" => Ok(Self::West),
            "southeast" => Ok(Self::Southeast),
            "southwest" => Ok(Self::Southwest),
            "northeast" => Ok(Self::Northeast),
            "northwest" => Ok(Self::Northwest),
            _ => Err(ParseHDirectionError),
        }
    }
}

/// Absolute position of a chunk.
///
/// The chunk position is indexable by [`HAxis`].
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ChunkPosition {
    /// the x component of the position.
    pub x: i32,
    /// the z component of the position.
    pub z: i32,
}

impl ChunkPosition {
    /// Returns a chunk position from the given coordinates.
    #[inline]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Returns the position where is `n` chunks south.
    ///
    /// This method is equivalent to `self.north(-n)`.
    #[inline]
    pub const fn south(mut self, n: i32) -> Self {
        self.z += n;
        self
    }

    /// Returns the position where is `n` chunks north.
    ///
    /// This method is equivalent to `self.south(-n)`.
    #[inline]
    pub const fn north(mut self, n: i32) -> Self {
        self.z -= n;
        self
    }

    /// Returns the position where is `n` chunks east.
    ///
    /// This method is equivalent to `self.west(-n)`.
    #[inline]
    pub const fn east(mut self, n: i32) -> Self {
        self.x += n;
        self
    }

    /// Returns the position where is `n` chunks west.
    ///
    /// This method is equivalent to `self.east(-n)`.
    #[inline]
    pub const fn west(mut self, n: i32) -> Self {
        self.x -= n;
        self
    }
}

impl fmt::Debug for ChunkPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("chunk")?;
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl fmt::Display for ChunkPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { x, z } = self;
        write!(f, "[{x}, {z}]")
    }
}

/// Absolute position of a block.
///
/// The block position is indexable by [`Axis`].
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct BlockPosition {
    /// the x component of the position.
    pub x: i32,
    /// the z component of the position.
    pub y: i32,
    /// the z component of the position.
    pub z: i32,
}

impl BlockPosition {
    /// Returns a block position from the given coordinates.
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Returns a block position from the chunk position and the block offset.
    #[inline]
    pub const fn from_parts(chunk: ChunkPosition, offset: BlockOffset) -> Self {
        const WIDTH: i32 = CHUNK_WIDTH as i32;

        Self {
            x: offset.x as i32 + chunk.x * WIDTH,
            y: offset.y as i32,
            z: offset.z as i32 + chunk.z * WIDTH,
        }
    }

    /// Returns the chunk position and the block offset in the chunk.
    #[inline]
    pub const fn into_parts(self) -> (ChunkPosition, BlockOffset) {
        const WIDTH: i32 = CHUNK_WIDTH as i32;

        /// Returns `(x.div_euclid(WIDTH), x.rem_euclid(WIDTH))`.
        ///
        /// Guarantees that `.1` ranges from `0` to `WIDTH - 1`.
        #[inline]
        const fn rem_div_width_euclid(x: i32) -> (i32, i32) {
            let (q, r) = (x / WIDTH, x % WIDTH);
            if r < 0 {
                (q - 1, r + WIDTH)
            } else {
                (q, r)
            }
        }

        let (qx, rx) = rem_div_width_euclid(self.x);
        let (qz, rz) = rem_div_width_euclid(self.z);

        (
            ChunkPosition::new(qx, qz),
            BlockOffset::new(rx as u8, self.y as u16, rz as u8),
        )
    }

    /// Returns the position where is `n` blocks south.
    ///
    /// This method is equivalent to `self.north(-n)`.
    #[inline]
    pub const fn south(mut self, n: i32) -> Self {
        self.z += n;
        self
    }

    /// Returns the position where is `n` blocks north.
    ///
    /// This method is equivalent to `self.south(-n)`.
    #[inline]
    pub const fn north(mut self, n: i32) -> Self {
        self.z -= n;
        self
    }

    /// Returns the position where is `n` blocks east.
    ///
    /// This method is equivalent to `self.west(-n)`.
    #[inline]
    pub const fn east(mut self, n: i32) -> Self {
        self.x += n;
        self
    }

    /// Returns the position where is `n` blocks west.
    ///
    /// This method is equivalent to `self.east(-n)`.
    #[inline]
    pub const fn west(mut self, n: i32) -> Self {
        self.x -= n;
        self
    }

    /// Returns the position where is `n` blocks up.
    ///
    /// This method is equivalent to `self.down(-n)`.
    #[inline]
    pub const fn up(mut self, n: i32) -> Self {
        self.y += n;
        self
    }

    /// Returns the position where is `n` blocks down.
    ///
    /// This method is equivalent to `self.up(-n)`.
    #[inline]
    pub const fn down(mut self, n: i32) -> Self {
        self.y -= n;
        self
    }
}

impl fmt::Debug for BlockPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("block")?;
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl fmt::Display for BlockPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { x, y, z } = self;
        write!(f, "[{x}, {y}, {z}]")
    }
}

impl From<BlockPosition> for Vector3<i32> {
    #[inline]
    fn from(value: BlockPosition) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Vector3<i32>> for BlockPosition {
    #[inline]
    fn from(value: Vector3<i32>) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
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

impl ops::Sub<Self> for BlockPosition {
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

/// Relative position of a block in the chunk.
#[repr(C, align(4))]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct BlockOffset {
    /// the x component of the block offset.
    pub x: u8,
    /// the z component of the block offset.
    pub z: u8,
    /// the z component of the block offset.
    pub y: u16,
}

impl BlockOffset {
    /// Returns a block offset from the given coordinates.
    #[inline]
    pub const fn new(x: u8, y: u16, z: u8) -> Self {
        Self { x, z, y }
    }
}

impl fmt::Debug for BlockOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("block")?;
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl fmt::Display for BlockOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { x, y, z } = self;
        write!(f, "[~{x}, ~{y}, ~{z}]")
    }
}

/// implements following traits for the positions:
/// - `Index<_>`,           `IndexMut<_>`;
/// - `AsRef<[i32]>`,       `AsMut<[i32]>`;
/// - `AsRef<[i32; _]>`,    `AsMut<[i32; _]>`
/// - `From<[i32; _]>`,     `Into<[i32; _]>`
/// - `Serialize`,          `Deserialize<'de>`
macro_rules! impl_position {
    ($POSITION:ident [$length:expr, $AXIS:ident $(,)?] => { $( $lower:ident: $upper:ident ),+ $(,)? } ) => {
        impl ops::Index<$AXIS> for $POSITION {
            type Output = i32;

            #[inline]
            fn index(&self, index: $AXIS) -> &Self::Output {
                match index { $( $AXIS::$upper => &self.$lower, )* }
            }
        }

        impl ops::IndexMut<$AXIS> for $POSITION {
            #[inline]
            fn index_mut(&mut self, index: $AXIS) -> &mut Self::Output {
                match index { $( $AXIS::$upper => &mut self.$lower, )* }
            }
        }

        impl AsRef<[i32]> for $POSITION {
            #[inline]
            fn as_ref(&self) -> &[i32] {
                <Self as AsRef<[i32; $length]>>::as_ref(self)
            }
        }

        impl AsMut<[i32]> for $POSITION {
            #[inline]
            fn as_mut(&mut self) -> &mut [i32] {
                <Self as AsMut<[i32; $length]>>::as_mut(self)
            }
        }

        impl AsRef<[i32; $length]> for $POSITION {
            #[inline]
            fn as_ref(&self) -> &[i32; $length] {
                // SAFETY: the memory layout of the chunk position is same as the array.
                unsafe { std::mem::transmute(self) }
            }
        }

        impl AsMut<[i32; $length]> for $POSITION {
            #[inline]
            fn as_mut(&mut self) -> &mut [i32; $length] {
                // SAFETY: the memory layout of the chunk position is same as the array.
                unsafe { std::mem::transmute(self) }
            }
        }

        impl From<[i32; $length]> for $POSITION {
            #[inline]
            fn from([$( $lower, )*]: [i32; $length]) -> Self {
                Self { $( $lower, )* }
            }
        }

        impl From<$POSITION> for [i32; $length] {
            #[inline]
            fn from(value: $POSITION) -> Self {
                [$( value.$lower, )*]
            }
        }

        impl Serialize for $POSITION {
            #[inline]
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                <[i32; $length] as Serialize>::serialize(self.as_ref(), serializer)
            }
        }

        impl<'de> Deserialize<'de> for $POSITION {
            #[inline]
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                <[i32; $length] as Deserialize<'de>>::deserialize(deserializer).map(From::from)
            }
        }
    };
}

impl_position!(BlockPosition [3, Axis]  => {  x: X, y: Y, z: Z, });
impl_position!(ChunkPosition [2, HAxis] => {  x: X,       z: Z, });

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::coords::{Axis, Direction};

    const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

    const DIRECTIONS: [Direction; 6] = [
        Direction::South,
        Direction::North,
        Direction::East,
        Direction::West,
        Direction::Up,
        Direction::Down,
    ];

    #[test]
    fn test_direction_serde() {
        let deserialized = r#"["south", "north", "east", "west", "up", "down"]"#;
        let deserialized: [Direction; 6] = serde_json::from_str(deserialized).unwrap();
        assert_eq!(DIRECTIONS, deserialized);

        for dir in DIRECTIONS {
            assert_eq!(dir, Direction::from_str(dir.as_ref()).unwrap());
        }
    }

    #[test]
    fn test_axis_serde() {
        let deserialized = r#"["x", "y", "z"]"#;
        let deserialized: [Axis; 3] = serde_json::from_str(deserialized).unwrap();
        assert_eq!(AXES, deserialized);

        for dir in AXES {
            assert_eq!(dir, Axis::from_str(dir.as_ref()).unwrap());
        }
    }
}
