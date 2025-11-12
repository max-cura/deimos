use core::marker::PhantomData;

use crate::{
    Access, ArrayDataType, DataType, Read, Register, RegisterLongName, Safe, ScalarDataType, Write,
};

pub trait StaticRegisterBlock {
    const ADDRESS: usize;
}
pub struct StaticRegister<T, S, const O: usize, PR, PW> {
    _t: PhantomData<T>,
    _s: PhantomData<S>,
    _pr: PhantomData<PR>,
    _pw: PhantomData<PW>,
}
impl<T, S, const U: usize, PR, PW> Clone for StaticRegister<T, S, U, PR, PW> {
    fn clone(&self) -> Self {
        Self {
            _t: PhantomData::default(),
            _s: PhantomData::default(),
            _pr: PhantomData::default(),
            _pw: PhantomData::default(),
        }
    }
}
impl<T, S, const U: usize, PR, PW> Copy for StaticRegister<T, S, U, PR, PW> {}
impl<T, S, const U: usize, PR, PW> StaticRegister<T, S, U, PR, PW> {
    pub fn new() -> Self {
        Self {
            _t: PhantomData::default(),
            _s: PhantomData::default(),
            _pr: PhantomData::default(),
            _pw: PhantomData::default(),
        }
    }
}
impl<T: DataType + Copy, S: StaticRegisterBlock, const O: usize, PR: Access, PW: Access> Register
    for StaticRegister<T, S, O, PR, PW>
{
    type DataType = T;
}
impl<T: DataType + Copy, S: StaticRegisterBlock, const O: usize, PW: Access> Read
    for StaticRegister<T, S, O, Safe, PW>
{
    fn read(&self) -> <Self::DataType as DataType>::Value
    where
        Self::DataType: ScalarDataType,
    {
        let ptr = core::ptr::with_exposed_provenance_mut::<<Self::DataType as DataType>::Value>(
            S::ADDRESS + O,
        );
        unsafe { ptr.read_volatile() }
    }

    unsafe fn read_at_unchecked(self, index: usize) -> <Self::DataType as ArrayDataType>::Element
    where
        Self::DataType: ArrayDataType,
    {
        assert!(index < <Self::DataType as ArrayDataType>::LEN);
        let base_ptr = core::ptr::with_exposed_provenance_mut::<
            <Self::DataType as ArrayDataType>::Element,
        >(S::ADDRESS + O);
        let ptr = unsafe { base_ptr.add(index) };
        unsafe { ptr.read_volatile() }
    }
}
impl<T: DataType + RegisterLongName + Copy, S: StaticRegisterBlock, const O: usize, PR: Access>
    Write for StaticRegister<T, S, O, PR, Safe>
{
    type LongName = T;

    fn write(&self, value: <Self::DataType as DataType>::Value)
    where
        Self::DataType: ScalarDataType,
    {
        let ptr = core::ptr::with_exposed_provenance_mut::<<Self::DataType as DataType>::Value>(
            S::ADDRESS + O,
        );
        unsafe { ptr.write_volatile(value) }
    }

    unsafe fn write_at_unchecked(
        self,
        index: usize,
        value: <Self::DataType as ArrayDataType>::Element,
    ) where
        Self::DataType: ArrayDataType,
    {
        assert!(index < <Self::DataType as ArrayDataType>::LEN);
        let base_ptr = core::ptr::with_exposed_provenance_mut::<
            <Self::DataType as ArrayDataType>::Element,
        >(S::ADDRESS + O);
        let ptr = unsafe { base_ptr.add(index) };
        unsafe { ptr.write_volatile(value) }
    }
}
