use bitflags::bitflags;

bitflags! {
  pub struct Flags:u32 {
    const IS_ANNOTATION = 0 << 1;
    const IS_TAG = 0 << 2;
    const IS_TAG_END = 0 << 3;
    const IS_ATTRIBUTE = 0 << 4;
    const IS_STRING = 0 << 5;

    const HAS_TEXT = 0 << 6;
    const IGNORE_ONCE = 0 << 7;
  }
}

impl Flags {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}
