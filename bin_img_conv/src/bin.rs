use crate::{buffer::WritableRcRefCellVec, Img};

use num_traits::ToPrimitive;
use png::{BitDepth, ColorType};
use rand::RngCore;
use std::io::{self, BufRead, Write};

#[derive(Clone)]
pub struct Bin(Vec<u8>, pub BitDepth, pub ColorType);

impl Bin {
    pub fn new<R>(bin: &mut R) -> Result<Self, Box<dyn std::error::Error>>
    where
        R: BufRead,
    {
        let mut inner = Vec::new();
        lzma_rs::lzma_compress(bin, &mut inner)?;

        Ok(Self::new_raw(inner))
    }

    pub(crate) fn new_raw(bin: Vec<u8>) -> Self {
        Self(bin, BitDepth::Sixteen, ColorType::Rgba)
    }

    fn bit_depth_internal(&self) -> Result<u8, Box<dyn std::error::Error>> {
        match self.1 {
            BitDepth::Eight => Ok(8),
            BitDepth::Sixteen => Ok(16),
            _ => Err("Unsupported bit depth.".into()),
        }
    }

    fn colors_per_pixel(&self) -> Result<u8, Box<dyn std::error::Error>> {
        match self.2 {
            ColorType::Grayscale => Ok(1),
            ColorType::GrayscaleAlpha => Ok(2),
            ColorType::Rgb => Ok(3),
            ColorType::Rgba => Ok(4),
            _ => Err("Unsupported color type.".into()),
        }
    }

    fn colors_per_pixel_except_alpha(&self) -> Result<u8, Box<dyn std::error::Error>> {
        match self.2 {
            ColorType::Grayscale => Ok(1),
            ColorType::GrayscaleAlpha => Ok(1),
            ColorType::Rgb => Ok(3),
            ColorType::Rgba => Ok(3),
            _ => Err("Unsupported color type.".into()),
        }
    }

    fn color_type_internal(&self) -> Result<u8, Box<dyn std::error::Error>> {
        match self.2 {
            ColorType::Grayscale => Ok(0),
            ColorType::GrayscaleAlpha => Ok(4),
            ColorType::Rgb => Ok(2),
            ColorType::Rgba => Ok(6),
            _ => Err("Unsupported color type.".into()),
        }
    }
}

impl TryFrom<&Bin> for Vec<u8> {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &Bin) -> Result<Self, Self::Error> {
        let mut result = Vec::new();
        lzma_rs::lzma_decompress(&mut value.0.as_slice(), &mut result)?;

        Ok(result)
    }
}

impl TryFrom<Bin> for Img {
    type Error = Box<dyn std::error::Error>;

    fn try_from(input: Bin) -> Result<Self, Self::Error> {
        let bit_depth = input.bit_depth_internal()?;
        let color_type = input.color_type_internal()?;
        let colors_per_pixel_except_alpha = input.colors_per_pixel_except_alpha()?;
        let colors_per_pixel = input.colors_per_pixel()?;

        // 1ピクセル何バイトか計算する
        let bits_per_pixel = bit_depth * colors_per_pixel;
        if bits_per_pixel % 8 != 0 {
            return Err("The number of bits per pixel must be a multiple of 8.".into());
        }
        let bytes_per_pixel = u128::from(bits_per_pixel / 8);

        let input_len = u128::try_from(input.0.len())?;
        let output_len = input_len + 16; // paddingの長さの情報(16バイト)を追加

        // 最後の1ピクセル分のデータが足りないときに追加する必要のあるデータ量を計算
        let output_rem = output_len % bytes_per_pixel;
        let mut padding: u128 = if output_rem == 0 {
            0
        } else {
            bytes_per_pixel - output_rem
        };

        // 最低限必要なピクセル数を計算
        let output_pixels_min = match padding {
            0 => (output_len / bytes_per_pixel) + 1, // 管理データ用に1ピクセル空けておく
            _ => (output_len / bytes_per_pixel) + 2,
        };

        // 画像を正方形にしたときの1辺の長さとピクセル数を計算
        let output_side_ = dashu::Real::from(output_pixels_min)
            .with_precision(256)
            .value()
            .with_rounding::<dashu::float::round::mode::HalfEven>()
            .sqrt()
            .ceil()
            .to_u128()
            .unwrap();
        let output_pixels = output_side_.checked_pow(2).ok_or("Too big file.")?;
        padding += (output_pixels - output_pixels_min) * bytes_per_pixel; // 正方形にするために追加する必要のあるデータ量を計算

        // 型変換
        let output_side = u32::try_from(output_side_)?;

        // 出力フォーマットの設定
        let output = WritableRcRefCellVec::new();
        let mut encoder = png::Encoder::new(output.clone(), output_side, output_side);
        encoder.set_depth(input.1);
        encoder.set_color(input.2);
        encoder.set_compression(png::Compression::Fast);
        encoder.set_filter(png::FilterType::NoFilter);
        encoder.validate_sequence(true);
        let mut writer = encoder.write_header()?.into_stream_writer()?;

        // データを書き出し
        // 管理データ(元のカラーフォーマットを復元するための情報)
        let colorfmt = (bit_depth << 3) | color_type;
        let colorfmt = if bit_depth == 16 {
            let colorfmt = u16::from(colorfmt) * 257;
            colorfmt.to_be_bytes()
        } else {
            [colorfmt, 0]
        };
        let bytes_per_color = usize::from(bit_depth / 8);
        for _ in 0..colors_per_pixel_except_alpha {
            writer.write_all(&colorfmt[0..bytes_per_color])?;
        }
        if colors_per_pixel != colors_per_pixel_except_alpha {
            for _ in 0..bytes_per_color {
                writer.write_all(&[0xFF])?;
            }
        }

        writer.write_all(&padding.to_be_bytes())?; // paddingの大きさ
        io::copy(&mut input.0.as_slice(), &mut writer)?; // データ本体
        drop(input);

        // paddingの部分はランダムなデータで埋める
        let mut rng = rand::thread_rng();
        for _ in 0..padding.try_into()? {
            let mut tmp = [0];
            rng.fill_bytes(&mut tmp);
            writer.write_all(&tmp)?;
        }
        drop(rng);
        writer.finish()?;

        // 結果を得る
        let mut lock = output.0.borrow_mut();
        let mut result = Vec::new();
        std::mem::swap(&mut *lock, &mut result);

        Ok(Img::new(result))
    }
}
