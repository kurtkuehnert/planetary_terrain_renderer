use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderSet, extract_resource::ExtractResource,
        extract_resource::ExtractResourcePlugin, renderer::RenderDevice,
    },
};
use std::{env::current_dir, time::SystemTime};

pub struct MetalCapturePlugin;

impl Plugin for MetalCapturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameCapture>()
            .add_plugins(ExtractResourcePlugin::<FrameCapture>::default())
            .add_systems(Update, input_capture);

        app.sub_app_mut(RenderApp)
            .add_systems(Render, start_capture.in_set(RenderSet::Prepare))
            .add_systems(Render, stop_capture.in_set(RenderSet::Cleanup));
    }
}

#[derive(Clone, Default, Resource, ExtractResource)]
pub struct FrameCapture {
    pub(crate) capture: bool,
}

pub fn input_capture(input: Res<ButtonInput<KeyCode>>, mut capture: ResMut<FrameCapture>) {
    capture.capture = input.just_pressed(KeyCode::KeyC);
}

pub fn start_capture(capture: Res<FrameCapture>, device: Res<RenderDevice>) {
    if !capture.capture {
        return;
    }

    println!("Capturing frame");

    let capture = metal::CaptureDescriptor::new();
    capture.set_destination(metal::MTLCaptureDestination::GpuTraceDocument);
    capture.set_output_url(current_dir().unwrap().join("captures").join(format!(
            "capture_{}.gputrace",
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        )));
    unsafe {
        device
            .wgpu_device()
            .as_hal::<wgpu_core::api::Metal, _, ()>(|device| {
                capture.set_capture_device(&device.unwrap().raw_device().lock());
            })
    };

    metal::CaptureManager::shared()
        .start_capture(&capture)
        .or_else(|_| {
            println!("Failed to start capture");
            Ok::<(), String>(())
        })
        .unwrap();
}

pub fn stop_capture(capture: Res<FrameCapture>) {
    if !capture.capture {
        return;
    }

    metal::CaptureManager::shared().stop_capture();
}
