//! TODO: missing documentation

use std::fmt;
use std::hash::Hash;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// An specialized index that identifies blocks of different kinds and states.
///
/// The block id consists of two parts:
///
/// - *block index* represents the kind of the block.
///
/// - *state index* represents the state of the block.
#[repr(C, align(4))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BlockId {
    block: u16,
    state: u16,
}

impl BlockId {
    /// ID of the air block (`"minecrium:air"`). Also the [`Default`] value of [`BlockId`].
    pub const AIR: Self = Self::from_parts(0, 0);

    /// Returns the block id from the specified block index and state index.
    #[must_use]
    #[inline(always)]
    pub const fn from_parts(block: u16, state: u16) -> Self {
        Self { block, state }
    }

    /// Returns the block id as block index (`.0`) and state index (`.1`).
    #[must_use]
    #[inline(always)]
    pub const fn into_parts(self) -> (u16, u16) {
        (self.block, self.state)
    }
}

impl Default for BlockId {
    #[inline]
    fn default() -> Self {
        Self::AIR
    }
}

impl From<BlockId> for u32 {
    #[inline]
    fn from(value: BlockId) -> Self {
        ((value.block as u32) << u16::BITS) | (value.state as u32)
    }
}

impl From<u32> for BlockId {
    #[inline]
    fn from(value: u32) -> Self {
        Self {
            block: (value >> u16::BITS) as u16,
            state: value as u16,
        }
    }
}

impl fmt::Debug for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockId({}:{})", self.block, self.state)
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.block, self.state)
    }
}

impl Hash for BlockId {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        u32::from(*self).hash(state);
    }
}

impl Serialize for BlockId {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        u32::from(*self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BlockId {
    #[inline]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u32::deserialize(deserializer).map(Self::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::BlockId;

    #[test]
    fn test_block_id() {
        assert_eq!(std::mem::size_of::<BlockId>(), std::mem::size_of::<u32>());
        assert_eq!(std::mem::align_of::<BlockId>(), std::mem::align_of::<u32>());

        assert_eq!(0_u32, u32::from(BlockId::AIR));
    }
}
