use clap::Parser;

use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::codecs::png::PngDecoder;
use image::codecs::webp::WebPDecoder;
use image::{Frames, GenericImageView, ImageFormat, ImageResult};
use image::io::Reader as ImageReader;
use image::{AnimationDecoder, DynamicImage, Frame, ImageDecoder, Rgba};
use image::imageops::{self, FilterType};

use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Overwrite the original file.
    /// Ignored if an explicit output is defined.
    #[clap(verbatim_doc_comment)]
    #[arg(short, long, default_value_t = false)]
    in_place: bool,

    /// Only analyze the file and print the new size as `{width}x{height}`.
    /// This can be used if scaling shall be done with a different tool, e.g. ImageMagick:
    /// 
    /// if size=$(fix-pixelart -a image.gif); then
    ///     convert image.gif -scale "$size" scaled.gif
    /// fi
    #[clap(verbatim_doc_comment)]
    #[arg(short = 'a', long, default_value_t = false)]
    only_analyze: bool,

    /// Only analyze the first frame of an animation.
    /// This can lead to a big speed-up, but will create a 1x1 pixel image if the first frame is a blank screen.
    #[clap(verbatim_doc_comment)]
    #[arg(short = 'f', long, default_value_t = false)]
    only_analyze_first_frame: bool,

    /// Ignore pixels at the border of the image.
    /// Sometimes pixel art is framed in a border that doesn't fit the pixel grid.
    /// This uses an extremely primitive strategy to ignore those.
    #[clap(verbatim_doc_comment)]
    #[arg(short = 'b', long, default_value_t = false)]
    ignore_border: bool,

    /// Image to resize.
    #[arg()]
    input: OsString,

    /// Where to write the output.
    /// Can be a file name or a directory.
    /// [default: "{basename}.scaled.{ext}"]
    #[clap(verbatim_doc_comment)]
    #[arg(default_value = None)]
    output: Option<OsString>,
}

struct CurrentStride {
    color: Rgba<u8>,
    stride: u32,
}

#[inline]
fn get_smallest_stride(img: &DynamicImage, ignore_border: bool) -> u32 {
    let mut strides = HashSet::new();
    if !get_smallest_stride_phase1(img, &mut strides, ignore_border) {
        return 1;
    }
    get_smallest_stride_phase2(&mut strides)
}

fn get_smallest_stride_phase1(img: &DynamicImage, strides: &mut HashSet<u32>, ignore_border: bool) -> bool {
    let mut curr_y = (0..img.width()).map(|_| CurrentStride {
        color: Rgba([0, 0, 0, 0]),
        stride: 0,
    }).collect::<Vec<_>>();

    for y in 0..img.height() {
        let mut curr_x = CurrentStride {
            color: Rgba([0, 0, 0, 0]),
            stride: 0,
        };
        for x in 0..img.width() {
            let color = img.get_pixel(x, y);
            if color == curr_x.color {
                curr_x.stride += 1;
            } else {
                if !ignore_border || x > curr_x.stride {
                    if curr_x.stride == 1 {
                        return false;
                    }
                    if curr_x.stride > 0 && curr_x.color[3] > 0 {
                        strides.insert(curr_x.stride);
                    }
                }
                curr_x.stride = 1;
                curr_x.color  = color;
            }

            let curr_y = &mut curr_y[x as usize];
            if curr_y.color == color {
                curr_y.stride += 1;
            } else {
                if !ignore_border || y > curr_y.stride {
                    if curr_y.stride == 1 {
                        return false;
                    }
                    if curr_y.stride > 0 && curr_y.color[3] > 0 {
                        strides.insert(curr_y.stride);
                    }
                }
                curr_y.stride = 1;
                curr_y.color  = color;
            }
        }
        if !ignore_border {
            if curr_x.stride == 1 {
                return false;
            }
            if curr_x.stride > 0 && curr_x.color[3] > 0 {
                strides.insert(curr_x.stride);
            }
        }
    }

    if !ignore_border {
        for curr_y in &curr_y {
            if curr_y.stride == 1 {
                return false;
            }
            if curr_y.stride > 0 && curr_y.color[3] > 0 {
                strides.insert(curr_y.stride);
            }
        }
    }

    return true;
}

fn get_smallest_stride_phase2(strides: &HashSet<u32>) -> u32 {
    let mut strides = strides.iter().cloned().collect::<Vec<_>>();
    strides.sort();

    let mut iter = strides.iter().cloned();

    let Some(mut min_stride) = iter.next() else {
        return 1;
    };

    if min_stride == 0 {
        let Some(next_stride) = iter.next() else {
            return 1;
        };
        min_stride = next_stride;
    }

    if min_stride == 1 {
        return 1;
    }

    for other in iter {
        if other % min_stride != 0 {
            return 1;
        }
    }

    min_stride
}


fn get_smallest_stride_from_animation<'a>(frames: impl Iterator<Item=&'a DynamicImage>, ignore_border: bool) -> ImageResult<u32> {
    let mut strides = HashSet::new();
    for frame in frames {
        if !get_smallest_stride_phase1(frame, &mut strides, ignore_border) {
            return Ok(1);
        }
    }

    let min_stride = get_smallest_stride_phase2(&strides);

    Ok(min_stride)
}

fn resize_still_image(img: &DynamicImage, output_format: ImageFormat, args: Args) -> ImageResult<()> {
    let output = output_from(args.output, args.input.as_os_str(), args.in_place, output_format)?;
    let min_stride = get_smallest_stride(&img, args.ignore_border);
    if min_stride <= 1 {
        eprintln!("failed to detect pixel art scaling");
        std::process::exit(1);
    }
    let (width, height) = img.dimensions();
    let new_width  = width  / min_stride;
    let new_height = height / min_stride;
    if args.only_analyze {
        println!("{new_width}x{new_height}");
        return Ok(());
    }
    println!("resizing {width} x {height} -> {new_width} x {new_height}");
    let img = imageops::resize(img, new_width, new_height, FilterType::Nearest);
    img.write_to(&mut BufWriter::new(File::options().write(true).create(true).open(&output)?), output_format)?;
    println!("written {output:?}");
    Ok(())
}

fn output_from(output: Option<OsString>, input: &OsStr, in_place: bool, format: ImageFormat) -> ImageResult<OsString> {
    if in_place {
        return Ok(input.to_owned());
    }

    let mut parent_dir = None;
    if let Some(output) = output {
        let path = Path::new(&output);
        let meta = path.metadata();
        match meta {
            Err(err) => {
                if err.kind() != std::io::ErrorKind::NotFound {
                    return Err(err.into());
                }
            }
            Ok(meta) => {
                if !meta.is_dir() {
                    return Ok(output);
                }
            }
        }
        parent_dir = Some(PathBuf::from(output));
    }

    let input_path = Path::new(input);
    let mut output = OsString::new();

    if let Some(parent) = parent_dir.as_deref().or_else(|| input_path.parent()) {
        let parent = parent.as_os_str();
        if !parent.is_empty() {
            output.push(parent);
            output.push(std::path::MAIN_SEPARATOR.to_string());
        }
    }

    if let Some(stem) = input_path.file_stem() {
        output.push(stem);
    } else {
        output.push("pixelart");
    }
    output.push(".scaled.");
    output.push(format.extensions_str()[0]);

    Ok(output)
}

fn resize_as_animated_gif(width: u32, height: u32, input_frames: Frames, args: Args) -> ImageResult<()> {
    let mut frames = Vec::new();
    for frame in input_frames {
        let frame: Frame = frame?;
        frames.push((frame.delay(), frame.left(), frame.top(), DynamicImage::from(frame.into_buffer())));
    }
    let min_stride = if args.only_analyze_first_frame {
        if let Some((_, _, _, img)) = frames.iter().next() {
            get_smallest_stride(img, args.ignore_border)
        } else {
            0
        }
    } else {
        get_smallest_stride_from_animation(frames.iter().map(|(_, _, _, img)| img), args.ignore_border)?
    };
    if min_stride <= 1 {
        eprintln!("failed to detect pixel art scaling");
        std::process::exit(1);
    }

    let new_width = width / min_stride;
    let new_height = height / min_stride;
    if args.only_analyze {
        println!("{new_width}x{new_height}");
        return Ok(());
    }

    println!("resizing {width} x {height} -> {new_width} x {new_height}");
    let output = output_from(args.output, args.input.as_os_str(), args.in_place, ImageFormat::Gif)?;
    let writer = BufWriter::new(File::options().write(true).create(true).open(&output)?);
    let mut encoder = GifEncoder::new(writer);
    if frames.len() > 1 {
        // XXX: the image crate doesn't support reading the repeat and speed parameters of animated GIFs!
        encoder.set_repeat(Repeat::Infinite)?;
    }
    for (delay, left, top, img) in frames {
        let buffer = imageops::resize(&img, img.width() / min_stride, img.height() / min_stride, FilterType::Nearest);
        encoder.encode_frame(Frame::from_parts(buffer, left / min_stride, top / min_stride, delay))?;
    }
    println!("written {output:?}");
    Ok(())
}

fn print_animation_downgrade_warning_if_needed(output_format: ImageFormat) {
    match output_format {
        ImageFormat::Png => {
            print_warning("PNG");
        }
        ImageFormat::WebP => {
            print_warning("WebP");
        }
        ImageFormat::Gif => {}
        _ => {
            // If this happens there is a new animated format that I only handled in some part of the code.
            let format_name = output_format.extensions_str()[0].to_ascii_uppercase();
            print_warning(&format_name);
        }
    }

    fn print_warning(format_name: &str) {
        eprintln!("animated {format_name} images are not supported, writing still image instead");
    }
}

fn resize_animation<'a>(decoder: impl AnimationDecoder<'a> + ImageDecoder, output_format: ImageFormat, args: Args) -> ImageResult<()> {
    let (width, height) = decoder.dimensions();
    if output_format == ImageFormat::Gif {
        resize_as_animated_gif(width, height, decoder.into_frames(), args)?;
    } else {
        if !args.only_analyze {
            print_animation_downgrade_warning_if_needed(output_format);
        }
        resize_still_image(&DynamicImage::from_decoder(decoder)?, output_format, args)?;
    }
    Ok(())
}

fn main() -> ImageResult<()> {
    let args = Args::parse();

    let output_format = if let Some(output) = &args.output {
        ImageFormat::from_path(output).ok()
    } else {
        None
    };

    let reader = ImageReader::open(&args.input)?.with_guessed_format()?;
    let maybe_format = reader.format();
    let output_format = output_format.unwrap_or(maybe_format.unwrap_or(ImageFormat::Png));

    match maybe_format {
        Some(ImageFormat::Gif) => {
            let decoder = GifDecoder::new(reader.into_inner())?;
            resize_animation(decoder, output_format, args)?;
        }
        Some(ImageFormat::WebP) => {
            let decoder = WebPDecoder::new(reader.into_inner())?;
            if decoder.has_animation() {
                resize_animation(decoder, output_format, args)?;
            } else {
                resize_still_image(&DynamicImage::from_decoder(decoder)?, output_format, args)?;
            }
        }
        Some(ImageFormat::Png) => {
            let decoder = PngDecoder::new(reader.into_inner())?;
            if decoder.is_apng()? {
                let (width, height) = decoder.dimensions();
                if output_format == ImageFormat::Gif {
                    resize_as_animated_gif(width, height, decoder.apng()?.into_frames(), args)?;
                } else {
                    if !args.only_analyze {
                        print_animation_downgrade_warning_if_needed(output_format);
                    }
                    resize_still_image(&DynamicImage::from_decoder(decoder)?, output_format, args)?;
                }
            } else {
                resize_still_image(&DynamicImage::from_decoder(decoder)?, output_format, args)?;
            }
        }
        _ => {
            let img = reader.decode()?;
            resize_still_image(&img, output_format, args)?;
        }
    }

    Ok(())
}
