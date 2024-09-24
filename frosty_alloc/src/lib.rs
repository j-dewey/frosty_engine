mod access;
mod allocator;
mod chunk;
mod frosty_box;
mod interim;

pub use access::*;
pub use allocator::Allocator;

/*
*  Object Lifetime:
*      |   Clone Pathway   |           |   Alloc Init Pathway  |
*      |-------------------|           |-----------------------|   * This pathway returns a ptr
*      |   1) Obj Init on  |           |   1) Viable memory    |      to a variable which may or
*      |       stack       |           |       region declared |      may not have valid values.
*      |   2) Obj clone    |           |       FrostyBox<T>    |      The ptr should not be used
*      |       into Alloc  |           |   2) Intermediate ptr |      until the values are init
*      |   3) Intermediate |           |       init            |
*      |       ptr init    |           |   3) ObjectHandle*    |      [ObjectHandle]
*      |   4) ObjectHandle |           |       returned to     |           |
*      |       returned to |           |       caller          |           V
*      |       caller      |           |   4) Caller updates   |      [InterimPtr] -> Also contains flags
*      ---------------------           |       values thru ptr |           |          abt underlying data
*                |                     -------------------------           V
*                |                                  |                 [FrostyBox]
*                ------------------------------------
*                                 |
*                                 V
*          |              Primary Lifetime                     |
*          |---------------------------------------------------|
*          |   no specific order, butaAll these could occur    |
           |                                                   |
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

pub unsafe trait FrostyAllocatable {
    fn id() -> AllocId
    where
        Self: Sized;
}

macro_rules! impl_alloc {
    ($obj:ident, $val:expr) => {
        unsafe impl FrostyAllocatable for $obj {
            fn id() -> AllocId
            where
                Self: Sized,
            {
                AllocId::new($val)
            }
        }
    };
}

// impl for some primatives
impl_alloc!(u8, 0);
impl_alloc!(u16, 1);
impl_alloc!(u32, 2);
impl_alloc!(u64, 3);
impl_alloc!(u128, 4);
impl_alloc!(usize, 5);
impl_alloc!(i8, 6);
impl_alloc!(i16, 7);
impl_alloc!(i32, 8);
impl_alloc!(i64, 9);
impl_alloc!(i128, 10);
impl_alloc!(isize, 11);
impl_alloc!(f32, 12);
impl_alloc!(f64, 13);
impl_alloc!(bool, 14);
impl_alloc!(char, 15); // added for nice power of 2
