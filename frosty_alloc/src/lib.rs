mod access;
mod allocator;
mod chunk;
mod frosty_box;

pub use access::*;
pub use allocator::Allocator;

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
