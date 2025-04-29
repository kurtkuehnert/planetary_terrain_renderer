use bevy_terrain::prelude::*;
use bevy_terrain_preprocess::prelude::*;
use gdal::raster::GdalDataType;
use std::env::set_var;

fn main() {
    unsafe {
        if true {
            set_var("RAYON_NUM_THREADS", "0");
            // set_var("GDAL_NUM_THREADS", "ALL_CPUS");
        } else {
            set_var("RAYON_NUM_THREADS", "1");
            // set_var("GDAL_NUM_THREADS", "1");
        }
    }

    // let args = Cli {
    //     src_path: vec!["/Volumes/ExternalSSD/gebco/gebco_original.tif".into()],
    //     terrain_path: "assets/terrains/earth".into(),
    //     temp_path: None,
    //     overwrite: true,
    //     no_data: PreprocessNoData::Source,
    //     data_type: PreprocessDataType::DataType(GdalDataType::Float32),
    //     fill_radius: 0.0,
    //     create_mask: false,
    //     lod_count: None,
    //     attachment_label: AttachmentLabel::Height,
    //     texture_size: 512,
    //     border_size: 4,
    //     mip_level_count: 4,
    //     format: AttachmentFormat::R32F,
    // };

    // let args = Cli {
    //     src_path: vec!["/Volumes/ExternalSSD/truemarble/500m".into()],
    //     terrain_path: "assets/terrains/earth".into(),
    //     temp_path: None,
    //     overwrite: true,
    //     no_data: PreprocessNoData::NoData(0.0),
    //     data_type: PreprocessDataType::DataType(GdalDataType::UInt8),
    //     fill_radius: 0.0,
    //     create_mask: false,
    //     lod_count: Some(7),
    //     attachment_label: AttachmentLabel::Custom("albedo".to_string()),
    //     texture_size: 512,
    //     border_size: 2,
    //     mip_level_count: 4,
    //     format: AttachmentFormat::Rgb8U,
    // };

    // let args = Cli {
    //     src_path: vec!["/Volumes/ExternalSSD/argeo/LOS-99-00-01_LonLat_200m_argeo_warped_EPSG32631_negated.tiff".into()],
    //     terrain_path: "assets/terrains/los".into(),
    //     temp_path: None,
    //     overwrite: true,
    //     no_data: PreprocessNoData::Source,
    //     data_type: PreprocessDataType::DataType(GdalDataType::Float32),
    //     fill_radius: 32.0,
    //     create_mask: true,
    //     lod_count: None,
    //     attachment_label: AttachmentLabel::Height,
    //     texture_size: 512,
    //     border_size: 4,
    //     mip_level_count: 4,
    //     format: AttachmentFormat::R32F,
    // };

    // let args = Cli {
    //     src_path: vec!["/Volumes/ExternalSSD/argeo/20190816_4500_128_NPD_Z01_AD_009_Depth_1m_EPSG32631_argeo_negated_cog.tif".into()],
    //     terrain_path: "assets/terrains/npd".into(),
    //     temp_path: None,
    //     overwrite: true,
    //     no_data: PreprocessNoData::Source,
    //     data_type: PreprocessDataType::DataType(GdalDataType::Float32),
    //     fill_radius: 32.0,
    //     create_mask: true,
    //     lod_count: None,
    //     attachment_label: AttachmentLabel::Height,
    //     texture_size: 512,
    //     border_size: 4,
    //     mip_level_count: 4,
    //     format: AttachmentFormat::R32F,
    // };

    // let args = Cli {
    //     src_path: vec![
    //         "/Volumes/ExternalSSD/argeo/22054_UTSIRA_DTM_COMBO_50x50m_VER_01.tiff".into(),
    //     ],
    //     terrain_path: "assets/terrains/utsira".into(),
    //     temp_path: None,
    //     overwrite: true,
    //     no_data: PreprocessNoData::NoData(0.0),
    //     data_type: PreprocessDataType::DataType(GdalDataType::Float32),
    //     fill_radius: 32.0,
    //     create_mask: true,
    //     lod_count: None,
    //     attachment_label: AttachmentLabel::Height,
    //     texture_size: 512,
    //     border_size: 4,
    //     mip_level_count: 4,
    //     format: AttachmentFormat::R32F,
    // };

    // let args = Cli {
    //     src_path: vec!["/Volumes/ExternalSSD/argeo/02_Raster".into()],
    //     terrain_path: "assets/terrains/sas".into(),
    //     temp_path: None,
    //     overwrite: true,
    //     no_data: PreprocessNoData::NoData(0.0),
    //     data_type: PreprocessDataType::DataType(GdalDataType::Float32),
    //     fill_radius: 32.0,
    //     create_mask: true,
    //     lod_count: None,
    //     attachment_label: AttachmentLabel::Height,
    //     texture_size: 512,
    //     border_size: 4,
    //     mip_level_count: 4,
    //     format: AttachmentFormat::R32F,
    // };

    // let args = Cli {
    //     src_path: vec!["/Volumes/ExternalSSD/argeo/01_Raster/60062_L05_32631_sss-18_S02a_20180814_20CM.tiff".into()],
    //     terrain_path: "assets/terrains/sas".into(),
    //     temp_path: None,
    //     overwrite: true,
    //     no_data: PreprocessNoData::NoData(0.0),
    //     data_type: PreprocessDataType::DataType(GdalDataType::UInt8),
    //     fill_radius: 0.0,
    //     create_mask: false,
    //     lod_count: Some(16),
    //     attachment_label: AttachmentLabel::Custom("albedo".to_string()),
    //     texture_size: 512,
    //     border_size: 2,
    //     mip_level_count: 4,
    //     format: AttachmentFormat::Rgba8U,
    // };

    let args = Cli {
        src_path: vec!["/Volumes/ExternalSSD/saxony/hartenstein_dsm".into()],
        terrain_path: "assets/terrains/hartenstein".into(),
        temp_path: None,
        overwrite: true,
        no_data: PreprocessNoData::Source,
        data_type: PreprocessDataType::DataType(GdalDataType::Float32),
        fill_radius: 32.0,
        create_mask: true,
        lod_count: None,
        attachment_label: AttachmentLabel::Height,
        texture_size: 512,
        border_size: 4,
        mip_level_count: 4,
        format: AttachmentFormat::R32F,
    };

    // let args = Cli {
    //     src_path: vec!["/Volumes/ExternalSSD/saxony/hartenstein_dop".into()],
    //     terrain_path: "assets/terrains/hartenstein".into(),
    //     temp_path: None,
    //     overwrite: true,
    //     no_data: PreprocessNoData::NoData(0.0),
    //     data_type: PreprocessDataType::DataType(GdalDataType::UInt8),
    //     fill_radius: 32.0,
    //     create_mask: false,
    //     lod_count: Some(15),
    //     attachment_label: AttachmentLabel::Custom("albedo".to_string()),
    //     texture_size: 512,
    //     border_size: 2,
    //     mip_level_count: 4,
    //     format: AttachmentFormat::Rgb8U,
    // };

    // let args = Cli {
    //     src_path: vec!["/Volumes/ExternalSSD/scope_data/LOS-99-00-01_LonLat_200m_argeo_warped_EPSG32631_negated.tiff".into()],
    //     terrain_path: "/Volumes/ExternalSSD/tiles/scope".into(),
    //     temp_path: None,
    //     overwrite: true,
    //     no_data: PreprocessNoData::Source,
    //     data_type: PreprocessDataType::DataType(GdalDataType::Float32),
    //     fill_radius: 16.0,
    //     create_mask: true,
    //     lod_count: None,
    //     attachment_label: AttachmentLabel::Height,
    //     texture_size: 512,
    //     border_size: 2,
    //     mip_level_count: 1,
    //     format: AttachmentFormat::RF32,
    // };

    // let args = Cli {
    //     src_path: vec!["/Volumes/ExternalSSD/swiss_data/swiss_large.tif".into()],
    //     terrain_path: "/Volumes/ExternalSSD/tiles/swiss".into(),
    //     temp_path: None,
    //     overwrite: true,
    //     no_data: PreprocessNoData::NoData(10000.0),
    //     data_type: PreprocessDataType::DataType(GdalDataType::Float32),
    //     fill_radius: 32.0,
    //     create_mask: true,
    //     lod_count: None,
    //     attachment_label: AttachmentLabel::Height,
    //     texture_size: 512,
    //     border_size: 4,
    //     mip_level_count: 4,
    //     format: AttachmentFormat::R32F,
    // };

    let (src_dataset, mut context) = PreprocessContext::from_cli(args).unwrap();

    preprocess(src_dataset, &mut context);
}
