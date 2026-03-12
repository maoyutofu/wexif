use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::{generate_exif_with_ai, print_original_exif, write_exif_to_image, ExifData};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

pub async fn run_mcp_server() -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = tokio::io::BufReader::new(stdin);
    let mut line = String::new();

    eprintln!("wexif MCP server started");

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break; // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        eprintln!("Received: {}", trimmed);

        let response = match serde_json::from_str::<JsonRpcRequest>(trimmed) {
            Ok(request) => handle_request(request).await,
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: None,
                error: Some(JsonRpcError {
                    code: -32700,
                    message: format!("Parse error: {}", e),
                    data: None,
                }),
            },
        };

        let response_json = serde_json::to_string(&response)?;
        eprintln!("Sending: {}", response_json);
        stdout.write_all(response_json.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }

    Ok(())
}

async fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let result = match request.method.as_str() {
        "initialize" => handle_initialize(request.params),
        "tools/list" => handle_tools_list(),
        "tools/call" => handle_tool_call(request.params).await,
        _ => Err(anyhow::anyhow!("Unknown method: {}", request.method)),
    };

    match result {
        Ok(value) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(value),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: e.to_string(),
                data: None,
            }),
        },
    }
}

fn handle_initialize(_params: Option<Value>) -> Result<Value> {
    Ok(json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "wexif",
            "version": "0.1.0"
        }
    }))
}

fn handle_tools_list() -> Result<Value> {
    Ok(json!({
        "tools": [
            {
                "name": "read_exif",
                "description": "读取图片的 EXIF 信息",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "image_path": {
                            "type": "string",
                            "description": "图片文件路径"
                        }
                    },
                    "required": ["image_path"]
                }
            },
            {
                "name": "write_exif",
                "description": "将 EXIF 数据写入图片",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "input_path": {
                            "type": "string",
                            "description": "输入图片路径"
                        },
                        "output_path": {
                            "type": "string",
                            "description": "输出图片路径（可选，默认覆盖原图）"
                        },
                        "exif_json_path": {
                            "type": "string",
                            "description": "EXIF JSON 文件路径"
                        },
                        "enable_ai": {
                            "type": "boolean",
                            "description": "是否使用 AI 生成 EXIF 信息（默认 false）"
                        }
                    },
                    "required": ["input_path", "exif_json_path"]
                }
            },
            {
                "name": "generate_exif_with_ai",
                "description": "使用 AI 生成图片的 EXIF 信息",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "image_path": {
                            "type": "string",
                            "description": "图片文件路径"
                        },
                        "output_json_path": {
                            "type": "string",
                            "description": "输出 JSON 文件路径"
                        }
                    },
                    "required": ["image_path", "output_json_path"]
                }
            }
        ]
    }))
}

async fn handle_tool_call(params: Option<Value>) -> Result<Value> {
    let params = params.context("Missing params")?;
    let tool_name = params["name"]
        .as_str()
        .context("Missing tool name")?;
    let arguments = &params["arguments"];

    match tool_name {
        "read_exif" => handle_read_exif(arguments).await,
        "write_exif" => handle_write_exif(arguments).await,
        "generate_exif_with_ai" => handle_generate_exif_ai(arguments).await,
        _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    }
}

async fn handle_read_exif(args: &Value) -> Result<Value> {
    let image_path = args["image_path"]
        .as_str()
        .context("Missing image_path")?;

    // 直接调用并返回成功消息
    print_original_exif(image_path)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("已读取图片 {} 的 EXIF 信息", image_path)
        }]
    }))
}

async fn handle_write_exif(args: &Value) -> Result<Value> {
    let input_path = args["input_path"]
        .as_str()
        .context("Missing input_path")?;
    let output_path = args["output_path"]
        .as_str()
        .unwrap_or(input_path);
    let exif_json_path = args["exif_json_path"]
        .as_str()
        .context("Missing exif_json_path")?;
    let enable_ai = args["enable_ai"]
        .as_bool()
        .unwrap_or(false);

    // 如果启用 AI，先生成 EXIF JSON 文件
    if enable_ai {
        generate_exif_with_ai(input_path, exif_json_path)?;
    }

    let exif_data = ExifData::from_file(exif_json_path)
        .context("加载 EXIF JSON 文件失败")?;

    write_exif_to_image(input_path, output_path, &exif_data)
        .context("写入 EXIF 数据失败")?;

    let message = if enable_ai {
        format!("已使用 AI 生成 EXIF 信息并成功写入: {}", output_path)
    } else {
        format!("已成功将 EXIF 数据写入: {}", output_path)
    };

    Ok(json!({
        "content": [{
            "type": "text",
            "text": message
        }]
    }))
}

async fn handle_generate_exif_ai(args: &Value) -> Result<Value> {
    let image_path = args["image_path"]
        .as_str()
        .context("Missing image_path")?;
    let output_json_path = args["output_json_path"]
        .as_str()
        .context("Missing output_json_path")?;

    generate_exif_with_ai(image_path, output_json_path)?;

    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("已使用 AI 生成 EXIF 信息并保存到: {}", output_json_path)
        }]
    }))
}
