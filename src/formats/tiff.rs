use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    image::ImageLoaderError,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use bytemuck::cast_slice;
use std::io::Cursor;
use tiff::decoder::{Decoder, DecodingResult};

#[derive(Default)]
pub struct TiffLoader;
impl AssetLoader for TiffLoader {
    type Asset = Image;
    type Settings = ();
    type Error = ImageLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Image, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let mut decoder = Decoder::new(Cursor::new(bytes)).unwrap();

        let (width, height) = decoder.dimensions().unwrap();

        let data = match decoder.read_image().unwrap() {
            DecodingResult::U8(data) => cast_slice(&data).to_vec(),
            DecodingResult::U16(data) => cast_slice(&data).to_vec(),
            DecodingResult::U32(data) => cast_slice(&data).to_vec(),
            DecodingResult::U64(data) => cast_slice(&data).to_vec(),
            DecodingResult::F16(_) => panic!("TIFF F16 format is not supported in Bevy"),
            DecodingResult::F32(data) => cast_slice(&data).to_vec(),
            DecodingResult::F64(data) => cast_slice(&data).to_vec(),
            DecodingResult::I8(data) => cast_slice(&data).to_vec(),
            DecodingResult::I16(data) => cast_slice(&data).to_vec(),
            DecodingResult::I32(data) => cast_slice(&data).to_vec(),
            DecodingResult::I64(data) => cast_slice(&data).to_vec(),
        };

        let mut image = Image::new_uninit(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            TextureFormat::bevy_default(),
            RenderAssetUsages::MAIN_WORLD,
        );

        // Avoid Image::new size assert
        image.data = Some(data);

        Ok(image)
    }

    fn extensions(&self) -> &[&str] {
        &["tif", "tiff"]
    }
}
