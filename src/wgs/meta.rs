use binrw::{binrw, NullString};

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct Meta {
    pub name: NullString,
    pub texture_count: u8,
}

impl Meta {
    pub fn new(name: &str) -> Self {
        let name = NullString(name.as_bytes().to_vec());
        Self {
            name,
            texture_count: 0,
        }
    }
}
