use core::marker::PhantomData;

use crate::{DataType, Modify, Read, Register, RegisterLongName, Safe, Write};

pub struct DynamicRegister<T: DataType, R, W> {
    ptr: *mut T::Value,
    r: PhantomData<R>,
    w: PhantomData<W>,
}

impl<T: DataType, R, W> Clone for DynamicRegister<T, R, W> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            r: PhantomData::default(),
            w: PhantomData::default(),
        }
    }
}
impl<T: DataType, R, W> Copy for DynamicRegister<T, R, W> {}

impl<T: DataType, R, W> DynamicRegister<T, R, W> {
    pub fn from_ptr(ptr: *mut T::Value) -> Self {
        Self {
            ptr,
            r: PhantomData::default(),
            w: PhantomData::default(),
        }
    }
}

impl<T: DataType, R, W> Register for DynamicRegister<T, R, W> {
    type DataType = T;
}
impl<T: DataType, W> Read for DynamicRegister<T, Safe, W> {
    fn read(&self) -> <Self::DataType as DataType>::Value
    where
        Self::DataType: crate::ScalarDataType,
    {
        unsafe { self.ptr.read_volatile() }
    }

    unsafe fn read_at_unchecked(
        self,
        index: usize,
    ) -> <Self::DataType as crate::ArrayDataType>::Element
    where
        Self::DataType: crate::ArrayDataType,
    {
        unsafe {
            self.ptr
                .cast::<<Self::DataType as crate::ArrayDataType>::Element>()
                .add(index)
                .read_volatile()
        }
    }
}
impl<T: DataType + RegisterLongName, R> Write for DynamicRegister<T, R, Safe> {
    type LongName = T;

    fn write(&self, value: <Self::DataType as DataType>::Value)
    where
        Self::DataType: crate::ScalarDataType,
    {
        unsafe { self.ptr.write_volatile(value) }
    }

    unsafe fn write_at_unchecked(
        self,
        index: usize,
        value: <Self::DataType as crate::ArrayDataType>::Element,
    ) where
        Self::DataType: crate::ArrayDataType,
    {
        unsafe {
            self.ptr
                .cast::<<Self::DataType as crate::ArrayDataType>::Element>()
                .add(index)
                .write_volatile(value)
        }
    }
}
impl<T: DataType + RegisterLongName> Modify for DynamicRegister<T, Safe, Safe> {}
