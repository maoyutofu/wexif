use anyhow::{Context, Result};
use clap::Parser;
use wexif::{generate_exif_with_ai, print_original_exif, run_mcp_server, write_exif_to_image, ExifData};

#[derive(Parser, Debug)]
#[command(name = "wexif")]
#[command(about = "EXIF 信息重写工具", long_about = None)]
#[command(version)]
struct Args {
    /// 启动 MCP 服务器模式
    #[arg(long)]
    mcp: bool,

    /// 输入图片路径
    #[arg(short, long)]
    input: Option<String>,

    /// 输出图片路径（可选，默认覆盖原图）
    #[arg(short, long)]
    output: Option<String>,

    /// EXIF JSON 文件路径
    #[arg(short, long, default_value = "exif.json")]
    exif_json: String,

    /// 启用 AI 生成 EXIF
    #[arg(long)]
    enable_ai: bool,

    /// 重写 EXIF 信息（不指定则只打印原有 EXIF）
    #[arg(short = 'w', long)]
    write: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // MCP 服务器模式
    if args.mcp {
        return run_mcp_server().await;
    }

    // CLI 模式需要 input 参数
    let input = args.input.context("需要指定 --input 参数")?;

    // 打印照片原有的 EXIF 信息
    print_original_exif(&input)?;

    // 如果没有指定 -w 参数，只打印 EXIF 信息后退出
    if !args.write {
        return Ok(());
    }

    // 如果启用 AI，先生成 EXIF JSON 文件
    if args.enable_ai {
        generate_exif_with_ai(&input, &args.exif_json)?;
    }

    // 从 JSON 文件加载 EXIF 数据
    let exif_data = ExifData::from_file(&args.exif_json)
        .context("加载 EXIF JSON 文件失败")?;

    println!("\n已加载 EXIF 数据:");
    println!("  相机: {} {}", exif_data.make, exif_data.model);
    println!("  镜头: {}", exif_data.lens);
    println!("  拍摄时间: {}", exif_data.date_time_original);
    println!("  位置: {}", exif_data.location);

    // 确定输出路径
    let output_path = args.output.as_ref().unwrap_or(&input);

    // 写入 EXIF 数据到图片
    write_exif_to_image(&input, output_path, &exif_data)
        .context("写入 EXIF 数据失败")?;

    println!("\n✓ EXIF 数据已成功写入: {}", output_path);

    Ok(())
}
