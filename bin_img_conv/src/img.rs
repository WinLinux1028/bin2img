use crate::Bin;

use image::GenericImageView;
use std::{io::Cursor, ops::Deref};

#[derive(Clone)]
pub struct Img(Vec<u8>);

impl Img {
    pub fn new(img: Vec<u8>) -> Self {
        Self(img)
    }
}

impl From<Img> for Vec<u8> {
    fn from(mut val: Img) -> Self {
        let mut result = Vec::new();
        std::mem::swap(&mut val.0, &mut result);

        result
    }
}

impl TryFrom<&Img> for Bin {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &Img) -> Result<Self, Self::Error> {
        // 管理データを読み込む
        let mut img = image::load_from_memory(&value.0)?;
        let colorfmt = img.get_pixel(0, 0);
        let bit_depth = colorfmt[0] >> 3;
        let color_type = colorfmt[0] & 0b00000111;

        // 読み込んだデータに従って元のカラーフォーマットに変換する
        img = match (bit_depth, color_type) {
            (8, 0) => img.into_luma8().into(),
            (8, 2) => img.into_rgb8().into(),
            (8, 4) => img.into_luma_alpha8().into(),
            (8, 6) => img.into_rgba8().into(),
            (16, 0) => img.into_luma16().into(),
            (16, 2) => img.into_rgb16().into(),
            (16, 4) => img.into_luma_alpha16().into(),
            (16, 6) => img.into_rgba16().into(),
            _ => panic!("Unsupported color format."),
        };
        let mut input = Vec::new();
        let mut cursor = Cursor::new(&mut input);
        img.write_to(&mut cursor, image::ImageOutputFormat::Png)?;

        let decoder = png::Decoder::new(input.as_slice());
        let mut reader = decoder.read_info()?;
        let mut input = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut input)?;

        // データの長さをチェックする
        let info = reader.info();
        if input.len() <= info.bytes_per_pixel() + 16 {
            Err("Too short data.")?;
        }

        // 管理データを消す
        for _ in 0..(info.bytes_per_pixel()) {
            input.remove(0);
        }

        // paddingの長さを読み込む
        let padding = u128::from_be_bytes((&input[0..16]).try_into()?);
        let padding = usize::try_from(padding)?;

        // paddingの長さの情報を消す
        for _ in 0..16 {
            input.remove(0);
        }
        // paddingを消す
        for _ in 0..padding {
            input.pop();
        }

        let result = Bin::new_raw(input);
        Ok(result)
    }
}

impl Deref for Img {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
