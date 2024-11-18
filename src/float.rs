/// Convert a floating point number to the byte representation.
pub fn to_bit_repr(mut float: f32) -> [u8; 2] {
    let mut n = 0u16;
    let mut current = 327.68f32 * 2.0;
    for _ in 0..16 {
        n <<= 1;
        current /= 2.0;
        if float >= current - 0.005 {
            float -= current;
            n |= 1;
        }
    }
    n.to_be_bytes()
}

/// Conver a byte representation back to a floating point.
pub fn from_bit_repr(repr: [u8; 2]) -> f32 {
    let mut n = u16::from_be_bytes(repr);
    let mut current = 0.01;
    let mut result = 0.00;
    while n != 0 {
        if n & 1 == 1 {
            result += current;
        }
        current *= 2.0;
        n >>= 1;
    }
    result
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn roundtrip() {
      for i in 0..u16::MAX {
          let repr = i.to_be_bytes();
          let x = from_bit_repr(repr);
          let y = to_bit_repr(x);
          assert_eq!(repr, y, "{x}");
      }
  }
}
