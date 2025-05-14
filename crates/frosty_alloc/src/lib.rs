#![feature(unsize)]
#![feature(array_ptr_get)]
mod access;
mod allocator;
mod chunk;
mod frosty_box;
mod group;
mod handle;
mod interim;

pub mod debug;

pub use access::*;
pub use allocator::Allocator;
pub use group::*;
pub use handle::*;

/*
*  Object Lifetime:
*      |   Clone Pathway   |           |   Alloc Init Pathway  |
*      |-------------------|           |-----------------------|   * This pathway returns a ptr
*      |   1) Obj Init on  |           |   1) Viable memory    |      to a variable which may or
*      |       stack       |           |       region declared |      may not have valid values.
*      |   2) Obj clone    |           |       FrostyBox<T>    |      The ptr should not be used
*      |       into Alloc  |           |   2) Intermediate ptr |      until the values are init
*      |   3) Intermediate |           |       init            |
*      |       ptr init    |           |   3) ObjectHandle*    |      [AccessedData] -> automatically
*      |   4) ObjectHandle |           |       returned to     |           |            accesses/drops
*      |       returned to |           |       caller          |           V            values
*      |       caller      |           |   4) Caller updates   |      [ObjectHandle]
*      ---------------------           |       values thru ptr |           |
*                |                     -------------------------           V
*                |                                  |                 [InterimPtr] -> Holds metadata
*                ------------------------------------                      |          about FrostyBox
*                                 |                                        V
*                                 V                                  [FrostyBox]
*          |              Primary Lifetime                     |
*          |---------------------------------------------------|
*          |   no specific order, but All these could occur    |
*          |                                                   |
*          |   a)  ObjectHandle / ObjectHandleMut produced     |
*          |   b)  Data updated / Modified                     |
*          |   C)  Memory Freed                                |
*          _____________________________________________________
*                                 |
*                                (C)
*                                 |
*                                 V
*          |             Cleanup  Postmortem                   |
*          |---------------------------------------------------|
*          |   Issue:                                          |
*          |       Every ObjectHandle needs to be updated and  |
*          |       deleted to prevent incorrectly accesing     |
*          |       data                                        |
*          |   Solution*:                                      |    * This can slow down execution
*          |       InterimPtr is stored seperately from data   |      so unsafe access options
*          |       and holds a flag to indicate data has been  |      should exist
*          |       freed. ObjectHandle looks at InterimPtr     |
*          |       before accessing data.                      |
*          |   Alloc View:                                     |
*          |       1) Updated InterimPtr to show freed memory  |
*          |       2) Region returned to OrderedChunkList      |
*          |   Caller View:                                    |
*          |       1) ObjectHandle reads freed flag, fails to  |
*          |            return data                            |
*          |       2) ObjectHandle (hopefully)* dropped by     |    * even if it's not, the ptr
*          |            caller                                 |      will still fail to read
*          -----------------------------------------------------
*/

pub unsafe trait FrostyAllocatable: 'static {
    fn id() -> AllocId
    where
        Self: 'static + Sized,
    {
        AllocId::of::<Self>()
    }
}

macro_rules! impl_alloc {
    ($obj:ident) => {
        unsafe impl FrostyAllocatable for $obj {}
    };
}

macro_rules! impl_alloc_array {
    ($($N:literal)*) => (
    $(
        unsafe impl<T : FrostyAllocatable> FrostyAllocatable for [T; $N] {}
    )*
)}

// impl for some primatives
impl_alloc!(u8);
impl_alloc!(u16);
impl_alloc!(u32);
impl_alloc!(u64);
impl_alloc!(u128);
impl_alloc!(usize);
impl_alloc!(i8);
impl_alloc!(i16);
impl_alloc!(i32);
impl_alloc!(i64);
impl_alloc!(i128);
impl_alloc!(isize);
impl_alloc!(f32);
impl_alloc!(f64);
impl_alloc!(bool);
impl_alloc!(char);

impl_alloc_array!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 32 64 128 256 512);
