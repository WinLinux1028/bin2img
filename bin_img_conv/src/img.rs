use crate::Bin;

use image::GenericImageView;
use std::{io::Cursor, ops::Deref};

#[derive(Clone)]
pub struct Img<T: AsRef<[u8]>>(T);

impl<T: AsRef<[u8]>> Img<T> {
    pub fn new(img: T) -> Self {
        Self(img)
    }
}

impl From<Img<Vec<u8>>> for Vec<u8> {
    fn from(val: Img<Vec<u8>>) -> Self {
        val.0
    }
}

impl<T: AsRef<[u8]>> TryFrom<Img<T>> for Bin {
    type Error = Box<dyn std::error::Error>;

    fn try_from(input: Img<T>) -> Result<Self, Self::Error> {
        // 管理データを読み込む
        let cursor = Cursor::new(input.0);
        let mut img = image::io::Reader::new(cursor);
        img.no_limits();
        let mut img = img.with_guessed_format()?.decode()?;

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
            _ => return Err("Unsupported color format.".into()),
        };

        // png内のデータを読み込む
        let color_type = img.color();
        let bytes_per_pixel = usize::from(color_type.bytes_per_pixel());
        let colors_per_pixel = usize::from(color_type.channel_count());
        let mut input = img.into_bytes();

        // バイトエンディアンを変換する
        if bytes_per_pixel / colors_per_pixel == 2 {
            let mut i = 0;
            while i < input.len() {
                unsafe {
                    let word: &u16 = std::mem::transmute(&input[i]);
                    let word = word.to_be_bytes();
                    *input.get_unchecked_mut(i) = word[0];
                    *input.get_unchecked_mut(i + 1) = word[1];
                }
                i += 2;
            }
        }

        // データの長さをチェックする
        if input.len() <= bytes_per_pixel + 16 {
            return Err("Too short data.".into());
        }

        // 管理データを消す
        for _ in 0..bytes_per_pixel {
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

        input.shrink_to_fit();
        let result = Bin::new_raw(input);
        Ok(result)
    }
}

impl<T: AsRef<[u8]>> Deref for Img<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
