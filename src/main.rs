/**
 * BMP2YUV on Rust
 * Author: RicePaste
 * 使用rust实现图像读取和颜色空间转换，
 * 将文件夹中的BMP图像读取，然后将RGB图像转换到YUV颜色空间并保存，
 * 不能调用现有的图像读取函数、颜色空间转换函数，代码自己编写。
 * 使用此代码生成的YUV图像格式为 **YUV 4:4:4 8-bit packed**，请务必使用对应的解析方式打开。
 */

use std::fs::{self, File};
use std::io::{self, Read, Write, Seek};
use std::path::Path;

#[repr(C)]
#[derive(Debug)]
struct BMPFileHeader {
    file_type: u16,
    file_size: u32,
    reserved1: u16,
    reserved2: u16,
    offset_data: u32,
}

#[repr(C)]
#[derive(Debug)]
struct BMPInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_pixels_per_meter: i32,
    y_pixels_per_meter: i32,
    colors_used: u32,
    colors_important: u32,
}

fn read_bmp_header(file: &mut File) -> io::Result<(BMPFileHeader, BMPInfoHeader)> {
    let mut buffer = [0u8; 54];
    file.read_exact(&mut buffer)?;

    let file_header = BMPFileHeader {
        file_type: u16::from_le_bytes([buffer[0], buffer[1]]),
        file_size: u32::from_le_bytes([buffer[2], buffer[3], buffer[4], buffer[5]]),
        reserved1: u16::from_le_bytes([buffer[6], buffer[7]]),
        reserved2: u16::from_le_bytes([buffer[8], buffer[9]]),
        offset_data: u32::from_le_bytes([buffer[10], buffer[11], buffer[12], buffer[13]]),
    };

    let info_header = BMPInfoHeader {
        size: u32::from_le_bytes([buffer[14], buffer[15], buffer[16], buffer[17]]),
        width: i32::from_le_bytes([buffer[18], buffer[19], buffer[20], buffer[21]]),
        height: i32::from_le_bytes([buffer[22], buffer[23], buffer[24], buffer[25]]),
        planes: u16::from_le_bytes([buffer[26], buffer[27]]),
        bit_count: u16::from_le_bytes([buffer[28], buffer[29]]),
        compression: u32::from_le_bytes([buffer[30], buffer[31], buffer[32], buffer[33]]),
        size_image: u32::from_le_bytes([buffer[34], buffer[35], buffer[36], buffer[37]]),
        x_pixels_per_meter: i32::from_le_bytes([buffer[38], buffer[39], buffer[40], buffer[41]]),
        y_pixels_per_meter: i32::from_le_bytes([buffer[42], buffer[43], buffer[44], buffer[45]]),
        colors_used: u32::from_le_bytes([buffer[46], buffer[47], buffer[48], buffer[49]]),
        colors_important: u32::from_le_bytes([buffer[50], buffer[51], buffer[52], buffer[53]]),
    };

    Ok((file_header, info_header))
}

fn rgb_to_yuv(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let y = 0.299 * r + 0.587 * g + 0.114 * b;
    let u = -0.14713 * r - 0.28886 * g + 0.436 * b;
    let v = 0.615 * r - 0.51499 * g - 0.10001 * b;

    (
        (y * 255.0).clamp(0.0, 255.0) as u8,
        ((u + 0.5) * 255.0).clamp(0.0, 255.0) as u8,
        ((v + 0.5) * 255.0).clamp(0.0, 255.0) as u8,
    )
}

fn convert_bmp_to_yuv(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let mut file = File::open(input_path)?;
    let (file_header, info_header) = read_bmp_header(&mut file)?;

    let width = info_header.width as usize;
    let height = info_header.height as usize;
    let bit_count = info_header.bit_count as usize;
    let row_size = ((width * bit_count + 31) / 32) * 4; // BMP 行填充到 4 字节对齐
    let mut pixels = vec![0u8; row_size * height];

    file.seek(io::SeekFrom::Start(file_header.offset_data as u64))?;
    file.read_exact(&mut pixels)?;

    let mut output_file = File::create(output_path)?;

    // BMP 图像数据的行是从底到顶存储的
    for y in 0..height {
        let bmp_y = height - 1 - y;
        let row_start = bmp_y * row_size;
        for x in 0..width {
            let idx = row_start + x * 4; // DWORD 对齐
            let b = pixels[idx];
            let g = pixels[idx + 1];
            let r = pixels[idx + 2];
            if y == 0 {
                print!("{} ", idx);
                println!("{} {} {}", r, g, b);
            }

            let (y, u, v) = rgb_to_yuv(r, g, b);
            output_file.write_all(&[y, u, v])?;
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let input_folder = Path::new("input_images");
    let output_folder = Path::new("output_yuv");

    if !output_folder.exists() {
        fs::create_dir(output_folder)?;
    }

    for entry in fs::read_dir(input_folder)? {
        let entry = entry?;
        let input_path = entry.path();
        if input_path.extension().unwrap_or_default() == "bmp" {
            let output_path = output_folder.join(input_path.file_stem().unwrap()).with_extension("yuv");
            convert_bmp_to_yuv(&input_path, &output_path)?;
            println!("Converted {} to YUV.", input_path.display());
        }
    }

    Ok(())
}
