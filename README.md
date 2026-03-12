# wexif

一个基于 Rust 的 EXIF 信息重写工具，支持从 JSON 文件读取 EXIF 数据并写入图片，可选集成 AI 生成 EXIF 信息。

## 功能特性

- 读取和显示图片的 EXIF 信息
- 从 JSON 文件读取 EXIF 数据并写入图片
- 支持常见 EXIF 字段：相机型号、镜头、拍摄参数、GPS 位置等
- 可选集成 AI（doubao-seed-skill）自动生成 EXIF 数据
- 支持覆盖原图或输出到新文件
- **支持 MCP (Model Context Protocol) 服务模式**

## 依赖要求

- Rust 1.70+
- doubao-seed-skill（用于 AI 生成 EXIF）
  - 手动下载：访问 [doubao-seed-skill Releases](https://github.com/maoyutofu/doubao-seed-skill/releases) 下载对应平台的版本并放入 PATH 目录

## 安装

### 方式一：从 GitHub Release 下载（推荐）

**AI Agent 自动安装指令**：

```bash
# 检测系统平台
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# 转换架构名称
case $ARCH in
  x86_64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) echo "不支持的架构: $ARCH"; exit 1 ;;
esac

# 转换操作系统名称
case $OS in
  darwin) TARGET="${ARCH}-apple-darwin"; EXT="tar.gz" ;;
  linux) TARGET="${ARCH}-unknown-linux-gnu"; EXT="tar.gz" ;;
  mingw*|msys*|cygwin*) TARGET="${ARCH}-pc-windows-msvc"; EXT="zip" ;;
  *) echo "不支持的操作系统: $OS"; exit 1 ;;
esac

# 获取最新版本
LATEST_URL=$(curl -s https://api.github.com/repos/maoyutofu/wexif/releases/latest | grep "browser_download_url.*${TARGET}.${EXT}" | cut -d '"' -f 4)

# 下载并安装
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"

if [ "$EXT" = "tar.gz" ]; then
  curl -L "$LATEST_URL" | tar xz
else
  curl -L -o wexif.zip "$LATEST_URL"
  unzip -o wexif.zip
  rm wexif.zip
fi

chmod +x wexif
echo "wexif 已安装到 $INSTALL_DIR/wexif"
echo "请确保 $INSTALL_DIR 在 PATH 中"
```

**注意**：请将 `YOUR_USERNAME` 替换为实际的 GitHub 用户名。

**手动下载**：

访问 [Releases 页面](https://github.com/YOUR_USERNAME/wexif/releases) 下载对应平台的二进制文件：

- macOS Intel: `wexif-x86_64-apple-darwin.tar.gz`
- macOS Apple Silicon: `wexif-aarch64-apple-darwin.tar.gz`
- Linux x64: `wexif-x86_64-unknown-linux-gnu.tar.gz`
- Linux ARM64: `wexif-aarch64-unknown-linux-gnu.tar.gz`
- Windows x64: `wexif-x86_64-pc-windows-msvc.zip`
- Windows ARM64: `wexif-aarch64-pc-windows-msvc.zip`

解压后将可执行文件放到 PATH 目录中（如 `~/.local/bin` 或 `/usr/local/bin`）。

### 方式二：从源码编译

```bash
cargo build --release
```

编译后的可执行文件位于 `target/release/wexif`

## 使用方法

### CLI 模式

#### 读取图片 EXIF 信息

```bash
wexif --input photo.jpg
```

#### 写入 EXIF 数据

```bash
# 使用预置的 exif.json
wexif --input photo.jpg --write

# 指定输出文件
wexif --input photo.jpg --output photo_with_exif.jpg --write

# 指定 EXIF JSON 文件
wexif --input photo.jpg --exif-json custom_exif.json --write
```

#### 使用 AI 生成 EXIF 数据

```bash
wexif --input photo.jpg --write --enable-ai
```

### MCP 服务模式

启动 MCP 服务器：

```bash
wexif --mcp
```

#### MCP 配置

在 Claude Desktop 或其他 MCP 客户端的配置文件中添加：

```json
{
  "mcpServers": {
    "wexif": {
      "command": "/path/to/wexif",
      "args": ["--mcp"],
      "env": {
        "ARK_API_KEY": "your-ark-api-key-here"
      }
    }
  }
}
```

**重要**: 如果需要使用 AI 生成 EXIF 功能，必须在 `env` 中配置 `ARK_API_KEY` 环境变量。

配置文件位置：
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

#### 可用的 MCP Tools

MCP 服务提供三个独立的工具函数，可以灵活组合使用：

##### 1. read_exif - 读取图片的 EXIF 信息

读取并显示图片中已有的 EXIF 元数据。

**参数：**
- `image_path` (string, 必需) - 图片文件路径

**示例：**
```json
{
  "name": "read_exif",
  "arguments": {
    "image_path": "/path/to/photo.jpg"
  }
}
```

##### 2. generate_exif_with_ai - 使用 AI 生成 EXIF 信息

分析图片内容，使用 AI 自动生成合适的 EXIF 数据并保存为 JSON 文件。

**参数：**
- `image_path` (string, 必需) - 图片文件路径
- `output_json_path` (string, 必需) - 输出 JSON 文件路径

**示例：**
```json
{
  "name": "generate_exif_with_ai",
  "arguments": {
    "image_path": "/path/to/photo.jpg",
    "output_json_path": "/path/to/generated_exif.json"
  }
}
```

**注意：** 需要在 MCP 配置中设置 `ARK_API_KEY` 环境变量。

##### 3. write_exif - 将 EXIF 数据写入图片

从 JSON 文件读取 EXIF 数据并写入图片文件。

**参数：**
- `input_path` (string, 必需) - 输入图片路径
- `output_path` (string, 可选) - 输出图片路径，默认覆盖原图
- `exif_json_path` (string, 必需) - EXIF JSON 文件路径

**示例：**
```json
{
  "name": "write_exif",
  "arguments": {
    "input_path": "/path/to/photo.jpg",
    "output_path": "/path/to/photo_with_exif.jpg",
    "exif_json_path": "/path/to/exif.json"
  }
}
```

#### MCP 调用流程

根据不同需求，可以选择以下工作流程：

**流程 1：读取现有 EXIF 信息**
```
read_exif(image_path) → 显示 EXIF 信息
```

**流程 2：手动编写 EXIF 并写入**
```
1. 手动创建 exif.json 文件
2. write_exif(input_path, exif_json_path) → 写入图片
```

**流程 3：AI 生成 EXIF 并写入（推荐）**
```
1. generate_exif_with_ai(image_path, output_json_path) → 生成 JSON
2. (可选) 编辑生成的 JSON 文件
3. write_exif(input_path, exif_json_path) → 写入图片
```

**流程 4：完整工作流**
```
1. read_exif(image_path) → 查看原始 EXIF
2. generate_exif_with_ai(image_path, "ai_exif.json") → AI 生成新 EXIF
3. write_exif(input_path, output_path, "ai_exif.json") → 写入新 EXIF
4. read_exif(output_path) → 验证写入结果
```

#### 使用示例

在 Claude Desktop 中使用 MCP 工具：

```
# 1. 读取图片 EXIF
请使用 read_exif 读取 /Users/me/photo.jpg 的 EXIF 信息

# 2. AI 生成 EXIF
请使用 generate_exif_with_ai 为 /Users/me/photo.jpg 生成 EXIF，保存到 /Users/me/exif.json

# 3. 写入 EXIF
请使用 write_exif 将 /Users/me/exif.json 的数据写入 /Users/me/photo.jpg

# 4. 一键处理（组合调用）
请帮我处理 /Users/me/photo.jpg：
1. 先用 AI 生成 EXIF 到 exif.json
2. 然后写入图片
3. 最后读取验证
```

## EXIF JSON 格式

`exif.json` 文件格式示例：

```json
{
  "GPSLongitude": "117.9431",
  "Model": "ILCE-7M4",
  "DateTimeOriginal": "2024:12:10 09:28:45",
  "Orientation": "1",
  "Lens": "FE 50mm F1.8",
  "FocalLength": "50mm",
  "ISOSpeedRatings": 100,
  "GPSLatitude": "30.1126",
  "ExposureCompensation": "+0.3EV",
  "Location": "中国安徽省黄山市黟县宏村镇徽派古村落",
  "Make": "SONY",
  "ExposureTime": "1/200s",
  "FNumber": "f/2.8",
  "WhiteBalance": "Auto"
}
```

## 命令行参数

- `--mcp`: 启动 MCP 服务器模式
- `-i, --input <PATH>`: 输入图片路径（CLI 模式必需）
- `-o, --output <PATH>`: 输出图片路径（可选，默认覆盖原图）
- `-e, --exif-json <PATH>`: EXIF JSON 文件路径（默认：exif.json）
- `-w, --write`: 重写 EXIF 信息（不指定则只读取显示）
- `--enable-ai`: 启用 AI 生成 EXIF 数据

## 示例

1. 读取图片 EXIF 信息：
```bash
wexif --input vacation.jpg
```

2. 使用预置 EXIF 数据重写图片：
```bash
wexif --input vacation.jpg --write
```

3. 生成新文件：
```bash
wexif --input original.jpg --output modified.jpg --write
```

4. 使用 AI 自动生成并写入 EXIF：
```bash
wexif --input photo.jpg --write --enable-ai
```

5. 启动 MCP 服务：
```bash
wexif --mcp
```

## 许可证

MIT
