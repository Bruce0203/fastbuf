use core::marker::PhantomData;

pub struct FastSlice<'a, T> {
    start: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> FastSlice<'a, T> {
    pub fn new() -> Self {
        Self::from([].as_slice())
    }

    pub fn next(&self) {}
}

impl<'a, T> From<&'a [T]> for FastSlice<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self {
            start: value.as_ptr(),
            end: value.as_ptr_range().end,
            _marker: PhantomData,
        }
    }
}
