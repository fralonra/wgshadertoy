use binrw::binrw;
use std::fmt;

#[binrw]
#[brw(little)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    #[br(count = width * height * 4)]
    pub data: Vec<u8>,
}

impl fmt::Debug for Texture {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Texture")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("data_len", &self.data.len())
            .finish()
    }
}

impl Texture {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        assert!((width * height * 4) as usize == data.len());

        Self {
            width,
            height,
            data,
        }
    }
}
