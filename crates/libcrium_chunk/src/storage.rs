use cgmath::Point3;

use libcrium_block::common::BlockId;
use libcrium_core::physics::CHUNK_WIDTH;

/// Volume of a chunk section, (`= 4096` in blocks).
const VOLUME: usize = CHUNK_WIDTH.pow(3);

#[repr(align(8))]
#[derive(Clone, Copy)]
enum Slot {
    Next { index: u16 },
    Cell { block: BlockId, count: u16 },
}

struct BlockStore {
    /// the vector of the slots.
    palette: Vec<Slot>,
    /// the number of [`Slot::Cell`] in the `self.palette`.
    ///
    /// Guarantees that `self.count <= self.palette.len()`.
    count: u16,
    /// the index to the first [`Slot::Next`] in the `self.palette`.
    ///
    /// There is no [`Slot::Next`] if `self.first >= self.palette.len()`.
    first: u16,
    /// blocks in the chunk section.
    store: Box<[u16; VOLUME]>,
}

impl BlockStore {
    pub fn get(&self, offset: Point3<u8>) -> BlockId {
        if let Some(index) = compress(offset) {
            let index = self.store[index];
            match self.palette[index as usize] {
                Slot::Cell { block, .. } => block,
                Slot::Next { .. } => unreachable!(),
            }
        } else {
            panic!("index `{offset:?}` out of bounds");
        }
    }

    pub fn set(&mut self, offset: Point3<u8>, block: BlockId) -> BlockId {
        if let Some(index) = compress(offset) {
            let index = self.store[index];
            let origin = match self.palette[index as usize] {
                Slot::Cell { block, .. } => block,
                Slot::Next { .. } => unreachable!(),
            };

            if origin == block {
                origin
            } else {


                

                todo!()
            }
        } else {
            panic!("index `{offset:?}` out of bounds");
        }
    }
}

impl Default for BlockStore {
    fn default() -> Self {
        Self {
            palette: Vec::new(),
            count: 0,
            first: 0,
            store: Box::new([0; VOLUME]),
        }
    }
}

const ZOFFSET: u32 = 0;
const XOFFSET: u32 = 4;
const YOFFSET: u32 = 8;

#[inline]
fn compress(offset: Point3<u8>) -> Option<usize> {
    (offset.x < 16 && offset.y < 16 && offset.z < 16).then(|| {
        ((offset.z as usize) << ZOFFSET)
            + ((offset.x as usize) << XOFFSET)
            + ((offset.y as usize) << YOFFSET)
    })
}

#[inline]
fn decompress(index: usize) -> Option<Point3<u8>> {
    (index < 0x1000).then(|| Point3 {
        x: ((index >> XOFFSET) & (CHUNK_WIDTH - 1)) as u8,
        y: ((index >> YOFFSET) & (CHUNK_WIDTH - 1)) as u8,
        z: ((index >> ZOFFSET) & (CHUNK_WIDTH - 1)) as u8,
    })
}
