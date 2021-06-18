use bitflags::bitflags;

bitflags! {
  pub struct Flags:u32 {
    const NONE = 0;
    const IS_ANNOTATION = 1 << 1;
    const IS_TAG = 1 << 2;
    const IS_TAG_END = 1 << 3;
    const IS_ATTRIBUTE = 1 << 4;
    const IS_STRING = 1 << 5;

    const HAS_TEXT = 1 << 6;
    const IGNORE_ONCE = 1 << 7;
  }
}

impl Flags {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}
