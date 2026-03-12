use anyhow::{Context, Result};
use std::fs::File;
use std::io::BufReader;

/// 读取并打印图片的原始 EXIF 信息
pub fn print_original_exif(image_path: &str) -> Result<()> {
    println!("=== 原始 EXIF 信息 ===");
    println!("图片路径: {}\n", image_path);

    let file = File::open(image_path)
        .context(format!("无法打开图片文件: {}", image_path))?;

    let mut reader = BufReader::new(file);
    let exif_reader = exif::Reader::new();

    match exif_reader.read_from_container(&mut reader) {
        Ok(exif) => {
            let mut has_exif = false;

            for field in exif.fields() {
                has_exif = true;

                // 尝试将字节序列转换为 UTF-8 字符串
                let display_value = match &field.value {
                    exif::Value::Byte(bytes) | exif::Value::Undefined(bytes, _) => {
                        String::from_utf8(bytes.clone())
                            .unwrap_or_else(|_| field.display_value().to_string())
                    }
                    exif::Value::Ascii(vec) => {
                        vec.iter()
                            .filter_map(|bytes| String::from_utf8(bytes.clone()).ok())
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                    _ => field.display_value().with_unit(&exif).to_string(),
                };

                println!("  {}: {}", field.tag, display_value);
            }

            if !has_exif {
                println!("  (未找到 EXIF 信息)");
            }
        }
        Err(e) => {
            println!("  无法读取 EXIF 信息: {}", e);
        }
    }

    println!("\n{}", "=".repeat(50));
    println!();

    Ok(())
}
