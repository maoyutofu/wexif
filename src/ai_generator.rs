use anyhow::{Context, Result};
use std::process::Command;

/// 使用 doubao-seed-skill 生成 EXIF JSON 文件
pub fn generate_exif_with_ai(image_path: &str, output_json: &str) -> Result<()> {
    let prompt_text = r#"解析图片,生成exif数据返回,包含位置信息，要求以json格式返回：`{"GPSLongitude":"","Model":"","DateTimeOriginal":"","Orientation":"","Lens":"","FocalLength":"","ISOSpeedRatings":0,"GPSLatitude":"","ExposureCompensation":"","Location":"","Make":"","ExposureTime":"","FNumber":"","WhiteBalance":""}`"#;

    println!("正在使用 AI 生成 EXIF 数据...");

    // 检查 ARK_API_KEY 环境变量
    let ark_api_key = std::env::var("ARK_API_KEY")
        .context("未设置 ARK_API_KEY 环境变量，请在 MCP 配置中设置")?;

    let output = Command::new("doubao-seed-skill")
        .arg("--image-url")
        .arg(image_path)
        .arg("--prompt")
        .arg(prompt_text)
        .arg("--output")
        .arg(output_json)
        .env("ARK_API_KEY", ark_api_key)
        .output()
        .context("执行 doubao-seed-skill 失败")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("doubao-seed-skill 执行失败: {}", stderr);
    }

    println!("AI 生成 EXIF 数据成功，已保存到: {}", output_json);

    Ok(())
}
