use serde::{Deserialize, Serialize};

use crate::strenum::strenum;

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

/// 8 kinds of horizontal directions, includes "south", "north", "east", "west", "southeast",
/// "southwest", "northeast", "northwest".
///
/// Also used as a block property that represents the face towards which the block is pointing.
///
/// See <https://hub.spigotmc.org/javadocs/spigot/org/bukkit/block/data/Directional.html>
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[strenum(crate = crate)]
pub enum HDirection {
    /// South, the direction parallel to the positive z axis.
    #[default]
    South = "south",
    /// Nouth, the direction parallel to the negative z axis.
    North = "north",
    /// East, the direction parallel to the positive x axis.
    East = "east",
    /// West, the direction parallel to the negative x axis.
    West = "west",
    /// Southeast, the direction between south and east.
    Southeast = "southeast",
    /// Southwest, the direction between south and west.
    Southwest = "southwest",
    /// Northeast, the direction between north and east.
    Northeast = "northeast",
    /// Northwest, the direction between north and west.
    Northwest = "northwest",
}

macro_rules! declare_error {
    ( $( #[ $meta:meta ] )* $vis:vis struct $Error:ident = $message:literal; ) => {
        $( #[ $meta ] )*
        $vis struct $Error;

        impl std::error::Error for $Error {}

        impl std::fmt::Display for $Error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str($message)
            }
        }
    };

}

declare_error! {
    /// An error that is [`<HDirection as TryFrom<Direction>>::Error`](TryFrom::Error).
    #[derive(Clone, Copy, Debug, Default)]
    pub struct HDirectionFromDirectionError = "expects horizontal direction `south`, `north`, `east` or `west`";
}

declare_error! {
    /// An error that is [`<Direction as TryFrom<HDirection>>::Error`](TryFrom::Error).
    #[derive(Clone, Copy, Debug, Default)]
    pub struct DirectionFromHDirection8Error = "expects direction `south`, `north`, `east` or `west`";
}

impl TryFrom<HDirection> for Direction {
    type Error = DirectionFromHDirection8Error;

    fn try_from(value: HDirection) -> Result<Self, Self::Error> {
        match value {
            HDirection::South => Ok(Direction::South),
            HDirection::North => Ok(Direction::North),
            HDirection::East => Ok(Direction::East),
            HDirection::West => Ok(Direction::West),
            _ => Err(DirectionFromHDirection8Error),
        }
    }
}

impl TryFrom<Direction> for HDirection {
    type Error = HDirectionFromDirectionError;

    fn try_from(value: Direction) -> Result<Self, Self::Error> {
        match value {
            Direction::South => Ok(HDirection::South),
            Direction::North => Ok(HDirection::North),
            Direction::East => Ok(HDirection::East),
            Direction::West => Ok(HDirection::West),
            _ => Err(HDirectionFromDirectionError),
        }
    }
}
