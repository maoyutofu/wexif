use serde::{Deserialize, Serialize};

/// EXIF 数据结构，对应 exif.json 文件格式
#[derive(Debug, Deserialize, Serialize)]
pub struct ExifData {
    #[serde(rename = "GPSLongitude")]
    pub gps_longitude: String,
    #[serde(rename = "Model")]
    pub model: String,
    #[serde(rename = "DateTimeOriginal")]
    pub date_time_original: String,
    #[serde(rename = "Orientation")]
    pub orientation: String,
    #[serde(rename = "Lens")]
    pub lens: String,
    #[serde(rename = "FocalLength")]
    pub focal_length: String,
    #[serde(rename = "ISOSpeedRatings")]
    pub iso_speed_ratings: u32,
    #[serde(rename = "GPSLatitude")]
    pub gps_latitude: String,
    #[serde(rename = "ExposureCompensation")]
    pub exposure_compensation: String,
    #[serde(rename = "Location")]
    pub location: String,
    #[serde(rename = "Make")]
    pub make: String,
    #[serde(rename = "ExposureTime")]
    pub exposure_time: String,
    #[serde(rename = "FNumber")]
    pub f_number: String,
    #[serde(rename = "WhiteBalance")]
    pub white_balance: String,
}

impl ExifData {
    /// 从 JSON 文件加载 EXIF 数据
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let data: ExifData = serde_json::from_str(&content)?;
        Ok(data)
    }
}
