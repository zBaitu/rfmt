impl<T, const N: usize> Index<usize> for StaticVec<T, {N}> {
  type Output = T;
  ///Asserts that `index` is less than the current length of the StaticVec,
  ///and if so returns the value at that position as a constant reference.
  #[inline(always)]
  fn index(&self, index: usize) -> &Self::Output {
    assert!(index < self.length);
    unsafe { self.data.get_unchecked(index).get_ref() }
  }
}  
