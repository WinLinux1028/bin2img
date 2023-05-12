#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{
    env,
    fs::File,
    io::{BufReader, Read, Write},
    path::PathBuf,
};

use bin_img_conv::{Bin, BitDepth, ColorType, Img};

struct Options {
    mode: fn(Options),
    color_type: ColorType,
    bit_depth: BitDepth,
    file_i: Option<PathBuf>,
    file_o: Option<PathBuf>,
}

fn main() {
    let mut options = Options {
        mode: help,
        color_type: ColorType::Rgb,
        bit_depth: BitDepth::Sixteen,
        file_i: None,
        file_o: None,
    };

    let mut args = env::args();
    args.next();

    let args: Vec<String> = args.collect();
    for (n, i) in args.iter().enumerate() {
        if n + 1 == args.len() {
            if options.file_i.is_some() {
                options.file_o = Some(PathBuf::from(i));
            } else {
                options.file_i = Some(PathBuf::from(i));
            }
            continue;
        }

        match i.as_str() {
            "-e" => options.mode = bin_to_img,
            "-d" => options.mode = img_to_bin,
            "--gray" => options.color_type = ColorType::Grayscale,
            "--graya" => options.color_type = ColorType::GrayscaleAlpha,
            "--rgb" => options.color_type = ColorType::Rgb,
            "--rgba" => options.color_type = ColorType::Rgba,
            "--8" => options.bit_depth = BitDepth::Eight,
            "--16" => options.bit_depth = BitDepth::Sixteen,
            _ => {
                if n + 2 == args.len() {
                    options.file_i = Some(PathBuf::from(i))
                }
            }
        }
    }

    (options.mode)(options);
}

fn bin_to_img(options: Options) {
    let file_i = match options.file_i {
        Some(s) => s,
        None => panic!("You have to specify input file."),
    };
    let file_o = match options.file_o {
        Some(s) => s,
        None => PathBuf::from("./output.png"),
    };

    let mut input = BufReader::new(File::open(file_i).unwrap());
    let mut output = File::create(file_o).unwrap();

    let mut input = Bin::new(&mut input).unwrap();
    input.1 = options.bit_depth;
    input.2 = options.color_type;

    let result = Img::try_from(input).unwrap();

    output.write_all(&result).unwrap();
}

fn img_to_bin(options: Options) {
    let file_i = match options.file_i {
        Some(s) => s,
        None => panic!("You have to specify input file."),
    };
    let file_o = match options.file_o {
        Some(s) => s,
        None => PathBuf::from("./output.bin"),
    };

    let mut input_data = Vec::new();
    File::open(file_i)
        .unwrap()
        .read_to_end(&mut input_data)
        .unwrap();

    let mut output = File::create(file_o).unwrap();

    let input = Img::new(input_data);
    let result = Bin::try_from(input).unwrap();
    let result: Vec<u8> = (&result).try_into().unwrap();

    output.write_all(&result).unwrap();
}

fn help(_: Options) {
    println!(
        "bin2img [options] input output
Options:
-e Convert binary to png.
-d Convert png to binary.

Color types:
--gray
--graya
--rgb
--rgba

Bit depthes:
--8
--16

RGB 16 is recommended for Twitter."
    )
}
