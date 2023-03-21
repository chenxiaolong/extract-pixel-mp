use {
    std::{
        cmp,
        fs::File,
        io::{self, Seek, Read, SeekFrom, Write},
        path::{Path, PathBuf},
    },
    anyhow::{anyhow, Context, Result},
    clap::Parser,
    xmp_toolkit::{OpenFileOptions, XmpFile, XmpMeta},
};

const NS_CONTAINER: &str = "http://ns.google.com/photos/1.0/container/";
const NS_ITEM: &str = "http://ns.google.com/photos/1.0/container/item/";

#[derive(Debug, Parser)]
struct Opts {
    /// Path to input motion photo file
    #[arg(short, long, value_parser, value_name = "FILE")]
    input: PathBuf,
    /// Path to output video file
    #[arg(short, long, value_parser, value_name = "FILE")]
    output: Option<PathBuf>,
    /// Output to stdout
    #[arg(short = 'O', long)]
    stdout: bool,
}

fn get_mp_size(path: &Path) -> Result<i64> {
    let mut file = XmpFile::new()?;
    file.open_file(path, OpenFileOptions::default())?;

    let xmp = file.xmp()
        .ok_or_else(|| anyhow!("File has no XMP data"))?;

    let containers_len = xmp.array_len(NS_CONTAINER, "Directory");

    for i in 1..=containers_len {
        let array_item_path = XmpMeta::compose_array_item_path(NS_CONTAINER, "Directory", i.try_into().unwrap())?;
        let item_path = XmpMeta::compose_struct_field_path(NS_CONTAINER, &array_item_path, NS_CONTAINER, "Item")?;
        let mime_path = XmpMeta::compose_struct_field_path(NS_CONTAINER, &item_path, NS_ITEM, "Mime")?;
        let length_path = XmpMeta::compose_struct_field_path(NS_CONTAINER, &item_path, NS_ITEM, "Length")?;
        let semantic_path = XmpMeta::compose_struct_field_path(NS_CONTAINER, &item_path, NS_ITEM, "Semantic")?;

        let semantic = xmp.property(NS_CONTAINER, &semantic_path)
            .ok_or_else(|| anyhow!("Directory item {i} has no semantic field"))?;
        if semantic.value != "MotionPhoto" {
            continue
        }

        let mime = xmp.property(NS_CONTAINER, &mime_path)
            .ok_or_else(|| anyhow!("Directory item {i} has no MIME type"))?;
        if mime.value != "video/mp4" {
            return Err(anyhow!("Unexpected MIME type for motion photo: {:#?}", mime.value));
        }

        let length = xmp.property_i64(NS_CONTAINER, &length_path)
            .ok_or_else(|| anyhow!("Directory item {i} has no length field"))?;

        return Ok(length.value);
    }

    Err(anyhow!("Motion photo directory item not found"))
}

fn copy_from_end<R: Read + Seek, W: Write>(input: &mut R, output: &mut W, mut length: i64) -> Result<()> {
    if length < 0 {
        return Err(anyhow!("Length is negative"));
    }

    input.seek(SeekFrom::End(-length))?;

    let mut buf = [0u8; 16384];

    while length > 0 {
        let to_read = cmp::min(length, buf.len() as i64) as usize;

        input.read_exact(&mut buf[..to_read])?;
        output.write_all(&buf[..to_read])?;

        length -= to_read as i64;
    }

    Ok(())
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    if opts.output.is_some() && opts.stdout {
        return Err(anyhow!("--output and --stdout cannot be used at the same time"));
    }

    let length = get_mp_size(&opts.input)
        .with_context(|| anyhow!("Failed to get embedded video size from: {:?}", opts.input))?;

    let mut input = File::open(&opts.input)
        .with_context(|| anyhow!("Failed to open for reading: {:?}", opts.input))?;

    if opts.stdout {
        let mut stdout = io::stdout().lock();
        copy_from_end(&mut input, &mut stdout, length)?;
    } else {
        let output_path = opts.output
            .unwrap_or_else(|| opts.input.with_extension("mp4"));
        let mut output = File::create(&output_path)
            .with_context(|| anyhow!("Failed to open for writing: {:?}", output_path))?;
        copy_from_end(&mut input, &mut output, length)?;
    }

    Ok(())
}
