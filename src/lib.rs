//! Access memory outside of virtual memory space.
//!
//! All applications live inside their own virtual memory space with mapped addresses. The `kernel` module allow to escape this space and access the memory directly and beyond the borders of the map. This can be useful for accessing otherwise inaccessible data by other applications or the operating system. Needles to say, this is highly unsafe and can lead to unstable systems if used irresponsible.
//!
//! **If virtual access is possible, it should strictly be preferred!**

#![no_std]

pub mod bindings;

use bindings as C;
use core::marker::PhantomData;
use core::mem::{self, MaybeUninit};
use wut::bindings as c_wut;

/// Access memory outside of virtual memory space.
#[derive(Debug, Clone, Copy)]
pub struct Physical<'a, T> {
    address: usize,
    _phantom: PhantomData<&'a T>,
}

// Implementation for generic lifetime
impl<'a, T> Physical<'a, T> {
    /// Create a physical memory access from a reference.
    ///
    /// # Safety
    ///
    /// This is the recommended way of using physical memory access as the borrow checker can ensure safety. However, not everything can be done within the save domain.
    ///
    /// For raw pointers refer to [from_ptr][Physical::from_ptr]. For fixed addresses refer to [from_address][Physical::from_physical].
    ///
    /// # Example
    ///
    /// ```
    /// let x = 1;
    /// let x = Physical::from_ref(&x);
    /// ```
    #[inline]
    pub fn from_ref(addr: &'a T) -> Physical<'a, T> {
        Self {
            address: Self::to_physical(addr as *const T as usize),
            _phantom: PhantomData,
        }
    }

    pub fn get_address(&self) -> usize {
        self.address
    }
}

// Separate implementation for static lifetime
impl<T> Physical<'static, T> {
    /// Create a physical memory access from a raw pointer.
    ///
    /// Prefer to use [from_ref][Physical::from_ref] wherever possible.
    ///
    /// # Safety
    ///
    /// The pointer must be valid, properly aligned, and point to initialized data. While operations will still work even if these conditions are not met, the data might be incomplete or corrupted.
    ///
    /// # Parameters
    ///
    /// - `ptr`: A raw pointer to the data.
    ///
    /// # Returns
    ///
    /// A `Physical` instance representing the physical memory access.
    ///
    /// # Example
    ///
    /// ```
    /// let x = vec![1, 2, 3];
    /// let y = Physical::from_ptr(x.as_ptr());
    /// ```
    #[inline]
    pub fn from_ptr(ptr: *const T) -> Physical<'static, T> {
        Self {
            address: Self::to_physical(ptr as usize),
            _phantom: PhantomData,
        }
    }

    ///Create a physical memory access from a memory address.
    ///
    /// Prefer to use [from_ref][Physical::from_ref] or [from_ptr][Physical::from_ptr] wherever possible.
    ///
    /// # Safety
    ///
    /// Address must be the location of valid, properly aligned, and initialized data. While operations will still work even if these conditions are not met, the data might be incomplete or corrupted.
    ///
    /// # Parameters
    ///
    /// - `physical_address`: A physical memory location.
    ///
    /// # Returns
    ///
    /// A `Physical` instance representing the physical memory access.
    ///
    /// # Example
    ///
    /// ```
    /// let x = Physical::<i32>::from_address(0xAABBCCDD);
    /// ```
    #[inline]
    pub fn from_address(physical_address: usize) -> Physical<'static, T> {
        Self {
            address: physical_address,
            _phantom: PhantomData,
        }
    }
}

impl<T> Physical<'_, T> {
    #[inline]
    fn to_physical(virtual_address: usize) -> usize {
        unsafe { c_wut::OSEffectiveToPhysical(virtual_address as u32) as usize }
    }

    #[inline]
    pub unsafe fn as_virtual_cached(&self) -> usize {
        c_wut::__OSPhysicalToEffectiveCached(self.address as u32) as usize
    }

    #[inline]
    pub unsafe fn as_virtual_uncached(&self) -> usize {
        c_wut::__OSPhysicalToEffectiveUncached(self.address as u32) as usize
    }

    #[inline]
    pub fn read(&self) -> T {
        let value = MaybeUninit::<T>::uninit();
        let mut ptr = Physical::from_ref(unsafe { &*value.as_ptr() });

        unsafe {
            copy(self, &mut ptr, mem::size_of::<T>());
            value.assume_init()
        }
    }

    #[inline]
    pub fn write(&mut self, value: T) {
        let ptr = Physical::from_ref(&value);

        unsafe {
            copy(&ptr, self, mem::size_of::<T>());
        }
    }

    #[inline]
    pub fn replace(&mut self, value: T) -> T {
        let prev = self.read();
        self.write(value);
        prev
    }
}

#[inline]
pub unsafe fn copy<T>(src: &Physical<T>, dst: &mut Physical<T>, count: usize) {
    C::KernelCopyData(dst.address as u32, src.address as u32, count as u32);
}
