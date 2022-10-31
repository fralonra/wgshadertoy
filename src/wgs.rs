mod meta;
mod texture;

use binrw::{binrw, BinRead, BinResult, BinWrite, NullString};
use meta::Meta;
use std::io;
use texture::Texture;

pub const DEFAULT_NAME: &'static str = "Untitled";
pub const EXTENSION: &'static str = "wgs";

#[derive(Debug)]
#[binrw]
#[brw(little)]
pub struct WgsData {
    meta: Meta,
    frag: NullString,
    #[br(count = meta.texture_count)]
    textures: Vec<Texture>,
}

impl WgsData {
    pub fn load(reader: &mut (impl io::Read + io::Seek)) -> BinResult<Self> {
        Self::read(reader)
    }

    pub fn new(name: &str, frag: &str) -> Self {
        let meta = Meta::new(name);
        let frag = NullString(frag.as_bytes().to_vec());
        Self {
            meta,
            frag,
            textures: vec![],
        }
    }

    pub fn add_texture(&mut self, width: u32, height: u32, data: Vec<u8>) {
        self.textures.push(Texture::new(width, height, data));
        self.meta.texture_count = self.textures.len() as u8;
    }

    pub fn change_texture(&mut self, index: usize, width: u32, height: u32, data: Vec<u8>) {
        self.textures[index] = Texture::new(width, height, data);
        self.meta.texture_count = self.textures.len() as u8;
    }

    pub fn frag(&self) -> String {
        self.frag.to_string()
    }

    pub fn name(&self) -> String {
        self.meta.name.to_string()
    }

    pub fn remove_texture(&mut self, index: usize) {
        self.textures.remove(index);
        self.meta.texture_count = self.textures.len() as u8;
    }

    pub fn save(&self, writer: &mut (impl io::Write + io::Seek)) -> BinResult<()> {
        self.write(writer)
    }

    pub fn set_frag(&mut self, frag: &str) {
        self.frag.0 = frag.as_bytes().to_vec();
    }

    pub fn set_name(&mut self, name: &str) {
        self.meta.name.0 = name.as_bytes().to_vec();
    }

    pub fn textures_ref(&self) -> &Vec<Texture> {
        &self.textures
    }
}
