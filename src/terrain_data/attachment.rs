use crate::math::TileCoordinate;
use bevy::render::render_resource::TextureFormat;
use bytemuck::cast_slice;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fmt::Error, str::FromStr};
use strum_macros::EnumIter;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum AttachmentLabel {
    #[default]
    Height,
    Custom(String), // Todo: this should not be a heap allocated string
    Empty(usize),
}

impl From<&AttachmentLabel> for String {
    fn from(value: &AttachmentLabel) -> Self {
        match value {
            AttachmentLabel::Height => "height".to_string(),
            AttachmentLabel::Custom(name) => name.clone(),
            AttachmentLabel::Empty(i) => format!("empty{i}").to_string(),
        }
    }
}

impl FromStr for AttachmentLabel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "height" => Ok(Self::Height),
            name => Ok(Self::Custom(name.to_string())),
        }
    }
}

/// The data format of an attachment.
#[derive(Serialize, Deserialize, EnumIter, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttachmentFormat {
    /// Three channels  8 bit unsigned integer
    Rgb8U,
    /// Four channels  8 bit unsigned integer
    Rgba8U,
    /// One channel  16 bit unsigned integer
    R16U,
    /// One channel  16 bit integer
    R16I,
    /// Two channels 16 bit unsigned integer
    Rg16U,
    /// One channel 32 bit float
    R32F,
}

impl FromStr for AttachmentFormat {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "rg8u" => Ok(Self::Rgb8U),
            "rgba8u" => Ok(Self::Rgba8U),
            "r16u" => Ok(Self::R16U),
            "r16i" => Ok(Self::R16I),
            "r32f" => Ok(Self::R32F),
            _ => Err(Error),
        }
    }
}

impl AttachmentFormat {
    pub(crate) fn render_format(self) -> TextureFormat {
        match self {
            AttachmentFormat::Rgb8U => TextureFormat::Rgba8UnormSrgb,
            AttachmentFormat::Rgba8U => TextureFormat::Rgba8UnormSrgb,
            AttachmentFormat::R16U => TextureFormat::R16Unorm,
            AttachmentFormat::Rg16U => TextureFormat::Rg16Unorm,
            AttachmentFormat::R32F => TextureFormat::R32Float,
            AttachmentFormat::R16I => TextureFormat::R16Snorm,
        }
    }

    pub(crate) fn processing_format(self) -> TextureFormat {
        match self {
            AttachmentFormat::Rgb8U => TextureFormat::Rgba8Unorm,
            AttachmentFormat::Rgba8U => TextureFormat::Rgba8Unorm,
            _ => self.render_format(),
        }
    }

    pub(crate) fn pixel_size(self) -> u32 {
        match self {
            AttachmentFormat::Rgb8U => 4,
            AttachmentFormat::Rgba8U => 4,
            AttachmentFormat::R16U => 2,
            AttachmentFormat::Rg16U => 4,
            AttachmentFormat::R32F => 4,
            AttachmentFormat::R16I => 2,
        }
    }
}

/// Configures an attachment.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttachmentConfig {
    /// The name of the attachment.
    pub texture_size: u32,
    /// The overlapping border size around the tile, used to prevent sampling artifacts.
    pub border_size: u32,
    pub mip_level_count: u32,
    /// The format of the attachment.
    pub format: AttachmentFormat,
}

impl Default for AttachmentConfig {
    fn default() -> Self {
        Self {
            texture_size: 512,
            border_size: 2,
            mip_level_count: 2,
            format: AttachmentFormat::R16U,
        }
    }
}

impl AttachmentConfig {
    pub fn center_size(&self) -> u32 {
        self.texture_size - 2 * self.border_size
    }

    pub fn offset_size(&self) -> u32 {
        self.texture_size - self.border_size
    }
}

#[derive(Clone)]
pub enum AttachmentData {
    /// Three channels  8 bit
    // Rgb8(Vec<(u8, u8, u8)>), Can not be represented currently
    /// Four  channels  8 bit
    Rgba8U(Vec<[u8; 4]>),
    /// One   channel  16 bit
    R16U(Vec<u16>),
    /// One   channel  16 bit
    R16I(Vec<i16>),
    /// Two   channels 16 bit
    Rg16U(Vec<[u16; 2]>),
    R32F(Vec<f32>),
}

impl AttachmentData {
    pub(crate) fn from_bytes(data: &[u8], format: AttachmentFormat) -> Self {
        match format {
            AttachmentFormat::Rgb8U => Self::Rgba8U(
                data.chunks(3)
                    .map(|chunk| [chunk[0], chunk[1], chunk[2], 255])
                    .collect_vec(),
            ),
            AttachmentFormat::Rgba8U => Self::Rgba8U(cast_slice(data).to_vec()),
            AttachmentFormat::R16U => Self::R16U(cast_slice(data).to_vec()),
            AttachmentFormat::R16I => Self::R16I(cast_slice(data).to_vec()),
            AttachmentFormat::Rg16U => Self::Rg16U(cast_slice(data).to_vec()),
            AttachmentFormat::R32F => Self::R32F(cast_slice(data).to_vec()),
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        match self {
            AttachmentData::Rgba8U(data) => cast_slice(data),
            AttachmentData::R16U(data) => cast_slice(data),
            AttachmentData::R16I(data) => cast_slice(data),
            AttachmentData::Rg16U(data) => cast_slice(data),
            AttachmentData::R32F(data) => cast_slice(data),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AttachmentTile {
    pub(crate) coordinate: TileCoordinate,
    pub(crate) label: AttachmentLabel,
}

#[derive(Clone)]
pub(crate) struct AttachmentTileWithData {
    pub(crate) atlas_index: u32,
    pub(crate) label: AttachmentLabel,
    pub(crate) data: AttachmentData,
}

/// An attachment of a [`TileAtlas`].
pub struct Attachment {
    pub(crate) path: PathBuf,
    pub(crate) texture_size: u32,
    pub(crate) center_size: u32,
    pub(crate) border_size: u32,
    pub(crate) mip_level_count: u32,
    pub(crate) format: AttachmentFormat,
}

impl Attachment {
    pub(crate) fn new(config: &AttachmentConfig, path: &str) -> Self {
        let path = if path.starts_with("assets") {
            path[7..].to_string()
        } else {
            path.to_string()
        };
        // let path = format!("assets/{path}/data/{name}");

        Self {
            path: PathBuf::from(path),
            texture_size: config.texture_size,
            center_size: config.center_size(),
            border_size: config.border_size,
            mip_level_count: config.mip_level_count,
            format: config.format,
        }
    }
}
