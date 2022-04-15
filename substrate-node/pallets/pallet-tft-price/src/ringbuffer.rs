//! # Transient RingBuffer implementation
//!
//! This pallet provides a trait and implementation for a ringbuffer that
//! abstracts over storage items and presents them as a FIFO queue.
//!
//! Usage Example:
//! ```rust, ignore
//! use ringbuffer::{RingBufferTrait, RingBufferTransient};
//!
//! // Trait object that we will be interacting with.
//! type RingBuffer = dyn RingBufferTrait<SomeStruct>;
//! // Implementation that we will instantiate.
//! type Transient = RingBufferTransient<
//!     SomeStruct,
//!     <TestModule as Store>::TestRange,
//!     <TestModule as Store>::TestMap,
//! >;
//! {
//!     let mut ring: Box<RingBuffer> = Box::new(Transient::new());
//!     ring.push(SomeStruct { foo: 1, bar: 2 });
//! } // `ring.commit()` will be called on `drop` here and syncs to storage
//! ```
//!
//! Note: You might want to introduce a helper function that wraps the complex
//! types and just returns the boxed trait object.
use core::marker::PhantomData;
use frame_support::storage::{IterableStorageMap, StorageValue};
use codec::{Codec, EncodeLike};
use sp_std::vec::Vec;

/// Trait object presenting the ringbuffer interface.
const MAX_LENGTH: u16 = 1440;
pub trait RingBufferTrait<Item>
where
    Item: Codec + EncodeLike,
{
    /// Store all changes made in the underlying storage.
    ///
    /// Data is not guaranteed to be consistent before this call.
    ///
    /// Implementation note: Call in `drop` to increase ergonomics.
    fn commit(&self);
    /// Push an item onto the end of the queue.
    fn push(&mut self, i: Item);
    /// Pop an item from the start of the queue.
    ///
    /// Returns `None` if the queue is empty.
    fn pop(&mut self) -> Option<Item>;
    /// Return whether the queue is empty.
    fn is_empty(&self) -> bool;

    fn get_all_values(&self) -> Vec<Item>;
}

// There is no equivalent trait in std so we create one.
pub trait WrappingOps {
    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
}

macro_rules! impl_wrapping_ops {
    ($type:ty) => {
        impl WrappingOps for $type {
            fn wrapping_add(self, rhs: Self) -> Self {
                self.wrapping_add(rhs)
            }
            fn wrapping_sub(self, rhs: Self) -> Self {
                self.wrapping_sub(rhs)
            }
        }
    };
}

impl_wrapping_ops!(u8);
impl_wrapping_ops!(u16);
impl_wrapping_ops!(u32);
impl_wrapping_ops!(u64);

/// Transient backing data that is the backbone of the trait object.
pub struct RingBufferTransient<Item, B, M>
where
    Item: Codec + EncodeLike,
    B: StorageValue<(u16, u16), Query = (u16, u16)>,
    M: IterableStorageMap<u16, Item, Query = Item>,
{
    start: u16,
    end: u16,
    _phantom: PhantomData<(Item, B, M)>,
}

impl<Item, B, M> RingBufferTransient<Item, B, M>
where
    Item: Codec + EncodeLike,
    B: StorageValue<(u16, u16), Query = (u16, u16)>,
    M: IterableStorageMap<u16, Item, Query = Item>,
{
    /// Create a new `RingBufferTransient` that backs the ringbuffer implementation.
    ///
    /// Initializes itself from the bounds storage `B`.
    pub fn new() -> RingBufferTransient<Item, B, M> {
        let (start, end) = B::get();
        RingBufferTransient {
            start,
            end,
            _phantom: PhantomData,
        }
    }
}

impl<Item, B, M> Drop for RingBufferTransient<Item, B, M>
where
    Item: Codec + EncodeLike,
    B: StorageValue<(u16, u16), Query = (u16, u16)>,
    M: IterableStorageMap<u16, Item, Query = Item>,
{
    /// Commit on `drop`.
    fn drop(&mut self) {
        <Self as RingBufferTrait<Item>>::commit(self);
    }
}

/// Ringbuffer implementation based on `RingBufferTransient`
impl<Item, B, M> RingBufferTrait<Item> for RingBufferTransient<Item, B, M>
where
    Item: Codec + EncodeLike,
    B: StorageValue<(u16, u16), Query = (u16, u16)>,
    M: IterableStorageMap<u16, Item, Query = Item>,
{
    /// Commit the (potentially) changed bounds to storage.
    fn commit(&self) {
        B::put((self.start, self.end));
    }

    /// Push an item onto the end of the queue.
    ///
    /// Will insert the new item, but will not update the bounds in storage.
    fn push(&mut self, item: Item) {
        M::insert(self.end, item);
        // this will intentionally overflow and wrap around when bonds_end
        // reaches `Index::max_value` because we want a ringbuffer.
        let mut next_index = self.end + 1;
        if next_index == MAX_LENGTH {
            next_index = 0;
        }
        if next_index == self.start {
            // queue presents as empty but is not
            // --> overwrite the oldest item in the FIFO ringbuffer
            self.start = self.start + 1;
            if self.start >= MAX_LENGTH {
                self.start = 0
            }
        }
        self.end = next_index;
    }

    /// Pop an item from the start of the queue.
    ///
    /// Will remove the item, but will not update the bounds in storage.
    fn pop(&mut self) -> Option<Item> {
        if self.is_empty() {
            return None;
        }
        let item = M::take(self.start);
        self.start = self.start + 1;
        if self.start >= MAX_LENGTH {
            self.start = 0;
        }

        item.into()
    }

    /// Return whether to consider the queue empty.
    fn is_empty(&self) -> bool {
        self.start == self.end
    }
    fn get_all_values(&self) -> Vec<Item> {
        let mut items: Vec<Item> = Vec::new();

        for (_, item) in M::iter() {
            items.push(item);
        }
        items
    }
}
