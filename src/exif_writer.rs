use crate::exif_data::ExifData;
use anyhow::{Context, Result};
use exif::{In, Tag, Value, Field};
use exif::experimental::Writer;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;

/// 解析 GPS 坐标字符串 (例如: "30°7′15″ N" 或 "117°58′22″ E")
fn parse_gps_coordinate(coord_str: &str) -> Result<(u32, u32, u32, u32, u32, u32)> {
    let parts: Vec<&str> = coord_str.split(['°', '′', '″']).collect();
    if parts.len() < 3 {
        anyhow::bail!("无效的 GPS 坐标格式: {}", coord_str);
    }

    let degrees = parts[0].trim().parse::<u32>()
        .with_context(|| format!("解析度数失败: {}", parts[0]))?;
    let minutes = parts[1].trim().parse::<u32>()
        .with_context(|| format!("解析分钟失败: {}", parts[1]))?;
    let seconds_str = parts[2].trim().split_whitespace().next().unwrap_or("0");
    let seconds = seconds_str.parse::<u32>()
        .with_context(|| format!("解析秒数失败: {}", seconds_str))?;

    Ok((degrees, 1, minutes, 1, seconds, 1))
}

/// 解析焦距字符串 (例如: "35mm")
fn parse_focal_length(focal_str: &str) -> Result<(u32, u32)> {
    let num_str = focal_str.trim().trim_end_matches("mm").trim();
    let focal = num_str.parse::<u32>()
        .with_context(|| format!("解析焦距失败: {}", focal_str))?;
    Ok((focal, 1))
}

/// 解析光圈字符串 (例如: "f/2.0")
fn parse_f_number(f_str: &str) -> Result<(u32, u32)> {
    let num_str = f_str.trim().trim_start_matches("f/").trim();
    let f_val = num_str.parse::<f32>()
        .with_context(|| format!("解析光圈失败: {}", f_str))?;
    let numerator = (f_val * 10.0) as u32;
    Ok((numerator, 10))
}

/// 解析曝光时间字符串 (例如: "1/250 s")
fn parse_exposure_time(exp_str: &str) -> Result<(u32, u32)> {
    let clean_str = exp_str.trim().trim_end_matches(" s").trim();
    if clean_str.contains('/') {
        let parts: Vec<&str> = clean_str.split('/').collect();
        if parts.len() == 2 {
            let numerator = parts[0].trim().parse::<u32>()?;
            let denominator = parts[1].trim().parse::<u32>()?;
            return Ok((numerator, denominator));
        }
    }
    anyhow::bail!("无效的曝光时间格式: {}", exp_str);
}

/// 解析曝光补偿字符串 (例如: "+0.3 EV")
fn parse_exposure_compensation(comp_str: &str) -> Result<(i32, i32)> {
    let num_str = comp_str.trim()
        .trim_end_matches(" EV")
        .trim_end_matches("EV")
        .trim();
    let comp_val = num_str.parse::<f32>()
        .with_context(|| format!("解析曝光补偿失败: {}", comp_str))?;
    let numerator = (comp_val * 10.0) as i32;
    Ok((numerator, 10))
}

/// 将 EXIF 数据写入图片文件
pub fn write_exif_to_image(image_path: &str, output_path: &str, exif_data: &ExifData) -> Result<()> {
    // 读取原始图片
    let input_path = Path::new(image_path);
    let _file = File::open(input_path)
        .with_context(|| format!("无法打开图片文件: {}", image_path))?;

    // 创建 Field 向量，确保生命周期足够长
    let mut fields: Vec<Field> = Vec::new();

    // 添加相机制造商
    if !exif_data.make.is_empty() {
        fields.push(Field {
            tag: Tag::Make,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![exif_data.make.as_bytes().to_vec()]),
        });
    }

    // 添加相机型号
    if !exif_data.model.is_empty() {
        fields.push(Field {
            tag: Tag::Model,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![exif_data.model.as_bytes().to_vec()]),
        });
    }

    // 添加镜头型号
    if !exif_data.lens.is_empty() {
        fields.push(Field {
            tag: Tag::LensModel,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![exif_data.lens.as_bytes().to_vec()]),
        });
    }

    // 添加拍摄日期时间
    if !exif_data.date_time_original.is_empty() {
        fields.push(Field {
            tag: Tag::DateTimeOriginal,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![exif_data.date_time_original.as_bytes().to_vec()]),
        });
    }

    // 添加 ISO
    if exif_data.iso_speed_ratings > 0 {
        fields.push(Field {
            tag: Tag::PhotographicSensitivity,
            ifd_num: In::PRIMARY,
            value: Value::Short(vec![exif_data.iso_speed_ratings as u16]),
        });
    }

    // 添加焦距
    if !exif_data.focal_length.is_empty() {
        if let Ok((num, den)) = parse_focal_length(&exif_data.focal_length) {
            fields.push(Field {
                tag: Tag::FocalLength,
                ifd_num: In::PRIMARY,
                value: Value::Rational(vec![exif::Rational { num, denom: den }]),
            });
        }
    }

    // 添加光圈
    if !exif_data.f_number.is_empty() {
        if let Ok((num, den)) = parse_f_number(&exif_data.f_number) {
            fields.push(Field {
                tag: Tag::FNumber,
                ifd_num: In::PRIMARY,
                value: Value::Rational(vec![exif::Rational { num, denom: den }]),
            });
        }
    }

    // 添加曝光时间
    if !exif_data.exposure_time.is_empty() {
        if let Ok((num, den)) = parse_exposure_time(&exif_data.exposure_time) {
            fields.push(Field {
                tag: Tag::ExposureTime,
                ifd_num: In::PRIMARY,
                value: Value::Rational(vec![exif::Rational { num, denom: den }]),
            });
        }
    }

    // 添加曝光补偿
    if !exif_data.exposure_compensation.is_empty() {
        if let Ok((num, den)) = parse_exposure_compensation(&exif_data.exposure_compensation) {
            fields.push(Field {
                tag: Tag::ExposureBiasValue,
                ifd_num: In::PRIMARY,
                value: Value::SRational(vec![exif::SRational { num, denom: den }]),
            });
        }
    }

    // 添加 GPS 纬度
    if !exif_data.gps_latitude.is_empty() {
        if let Ok((d1, d2, m1, m2, s1, s2)) = parse_gps_coordinate(&exif_data.gps_latitude) {
            fields.push(Field {
                tag: Tag::GPSLatitude,
                ifd_num: In::PRIMARY,
                value: Value::Rational(vec![
                    exif::Rational { num: d1, denom: d2 },
                    exif::Rational { num: m1, denom: m2 },
                    exif::Rational { num: s1, denom: s2 },
                ]),
            });

            let lat_ref = if exif_data.gps_latitude.contains('N') { b"N" } else { b"S" };
            fields.push(Field {
                tag: Tag::GPSLatitudeRef,
                ifd_num: In::PRIMARY,
                value: Value::Ascii(vec![lat_ref.to_vec()]),
            });
        }
    }

    // 添加 GPS 经度
    if !exif_data.gps_longitude.is_empty() {
        if let Ok((d1, d2, m1, m2, s1, s2)) = parse_gps_coordinate(&exif_data.gps_longitude) {
            fields.push(Field {
                tag: Tag::GPSLongitude,
                ifd_num: In::PRIMARY,
                value: Value::Rational(vec![
                    exif::Rational { num: d1, denom: d2 },
                    exif::Rational { num: m1, denom: m2 },
                    exif::Rational { num: s1, denom: s2 },
                ]),
            });

            let lon_ref = if exif_data.gps_longitude.contains('E') { b"E" } else { b"W" };
            fields.push(Field {
                tag: Tag::GPSLongitudeRef,
                ifd_num: In::PRIMARY,
                value: Value::Ascii(vec![lon_ref.to_vec()]),
            });
        }
    }

    // 添加位置描述
    if !exif_data.location.is_empty() {
        fields.push(Field {
            tag: Tag::ImageDescription,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![exif_data.location.as_bytes().to_vec()]),
        });
    }

    // 创建 Writer 并添加所有字段
    let mut exif_writer = Writer::new();
    for field in &fields {
        exif_writer.push_field(field);
    }

    // 将 EXIF 数据写入缓冲区
    let mut exif_buf = Cursor::new(Vec::new());
    exif_writer.write(&mut exif_buf, false)
        .context("写入 EXIF 数据失败")?;

    // 读取图片并重新写入带有新 EXIF 数据的图片
    let img = image::open(input_path)
        .with_context(|| format!("无法读取图片: {}", image_path))?;

    // 保存图片到输出路径
    img.save(output_path)
        .with_context(|| format!("无法保存图片到: {}", output_path))?;

    // 注意: image crate 保存时会丢失 EXIF 数据，需要手动注入
    inject_exif_to_jpeg(output_path, exif_buf.into_inner())?;

    Ok(())
}

/// 将 EXIF 数据注入到 JPEG 文件中
fn inject_exif_to_jpeg(jpeg_path: &str, exif_data: Vec<u8>) -> Result<()> {
    use std::io::{Read, Write};

    let mut file = File::open(jpeg_path)?;
    let mut jpeg_data = Vec::new();
    file.read_to_end(&mut jpeg_data)?;

    // 检查是否是 JPEG 文件
    if jpeg_data.len() < 2 || jpeg_data[0] != 0xFF || jpeg_data[1] != 0xD8 {
        anyhow::bail!("不是有效的 JPEG 文件");
    }

    // 构建新的 JPEG 数据
    let mut new_jpeg = Vec::new();
    new_jpeg.extend_from_slice(&[0xFF, 0xD8]); // SOI marker

    // 添加 APP1 (EXIF) 段
    new_jpeg.push(0xFF);
    new_jpeg.push(0xE1); // APP1 marker

    let exif_segment_size = (exif_data.len() + 8) as u16; // +8 for "Exif\0\0" and size
    new_jpeg.push((exif_segment_size >> 8) as u8);
    new_jpeg.push((exif_segment_size & 0xFF) as u8);
    new_jpeg.extend_from_slice(b"Exif\0\0");
    new_jpeg.extend_from_slice(&exif_data);

    // 跳过原始 JPEG 的 SOI 和任何现有的 APP 段
    let mut pos = 2;
    while pos < jpeg_data.len() - 1 {
        if jpeg_data[pos] == 0xFF {
            let marker = jpeg_data[pos + 1];
            if marker >= 0xE0 && marker <= 0xEF {
                // 跳过 APP 段
                let seg_len = ((jpeg_data[pos + 2] as u16) << 8) | (jpeg_data[pos + 3] as u16);
                pos += 2 + seg_len as usize;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // 添加剩余的 JPEG 数据
    new_jpeg.extend_from_slice(&jpeg_data[pos..]);

    // 写回文件
    let mut output = File::create(jpeg_path)?;
    output.write_all(&new_jpeg)?;

    Ok(())
}
