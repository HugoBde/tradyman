use std::fmt::Display;

pub struct RingBuffer<const N: usize> {
  data:   [usize; N],
  cursor: usize,
  sum:    usize,
}

impl<const N: usize> Default for RingBuffer<N> {
  fn default() -> Self {
    Self::new()
  }
}

impl<const N: usize> RingBuffer<N> {
  pub fn new() -> RingBuffer<N> {
    RingBuffer::<N> {
      data:   [0; N],
      cursor: 0,
      sum:    0,
    }
  }

  pub fn push(&mut self, val: usize) {
    self.sum -= self.data[self.cursor];
    self.sum += val;
    self.data[self.cursor] = val;
    self.cursor = (self.cursor + 1) % N;
  }

  pub fn average(&self) -> usize {
    self.sum / N
  }
}

impl<const N: usize> Display for RingBuffer<N> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[")?;
    for i in 0..N {
      write!(f, "{:<4} ", self.data[i])?;
    }
    writeln!(f, "]")
  }
}
