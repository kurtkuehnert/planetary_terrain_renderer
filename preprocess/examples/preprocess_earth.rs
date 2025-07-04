use bevy_terrain::prelude::*;
use bevy_terrain_preprocess::prelude::*;
use gdal::raster::GdalDataType;

fn main() {
    let args = Cli {
        src_path: vec!["source_data/gebco_earth.tif".into()],
        terrain_path: "assets/terrains/earth".into(),
        temp_path: None,
        overwrite: true,
        no_data: PreprocessNoData::Source,
        data_type: PreprocessDataType::DataType(GdalDataType::Float32),
        fill_radius: 0.0,
        create_mask: false,
        lod_count: None,
        attachment_label: AttachmentLabel::Height,
        texture_size: 512,
        border_size: 4,
        mip_level_count: 4,
        format: AttachmentFormat::R32F,
    };

    let (src_dataset, mut context) = PreprocessContext::from_cli(args).unwrap();

    preprocess(src_dataset, &mut context);

    let args = Cli {
        src_path: vec!["source_data/true_marble.tif".into()],
        terrain_path: "assets/terrains/earth".into(),
        temp_path: None,
        overwrite: true,
        no_data: PreprocessNoData::NoData(0.0),
        data_type: PreprocessDataType::DataType(GdalDataType::UInt8),
        fill_radius: 0.0,
        create_mask: false,
        lod_count: Some(4),
        attachment_label: AttachmentLabel::Custom("albedo".to_string()),
        texture_size: 512,
        border_size: 2,
        mip_level_count: 4,
        format: AttachmentFormat::Rgb8U,
    };

    let (src_dataset, mut context) = PreprocessContext::from_cli(args).unwrap();

    preprocess(src_dataset, &mut context);
}
