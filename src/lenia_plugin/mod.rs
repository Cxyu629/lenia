use std::num::NonZeroU32;

use bevy::core::cast_slice;
use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::{renderer::RenderQueue, RenderApp, RenderSet};

pub mod lenia_rules;
pub mod params;

use crate::*;

use self::params::LeniaGPUGrowthArrayBuffer;
use self::{
    lenia_rules::LeniaBoard,
    params::{LeniaGPUParams, LeniaGPUParamsBuffer, LeniaGPUTexture},
};

pub struct LeniaRenderPlugin {
    lenia_board: LeniaBoard,
}

impl LeniaRenderPlugin {
    pub fn new(lenia_board: LeniaBoard) -> Self {
        Self { lenia_board }
    }
}

impl Plugin for LeniaRenderPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        // Save kernel image for debugging purposes.
        let kernel_image = self.lenia_board.get_kernel_image();

        if let Err(e) = kernel_image.save_image("lenia/assets/kernels/kernel.png") {
            eprintln!("Image saving error: {}", e);
        };

        // Inserting params
        let params = self.lenia_board.generate_params();

        render_app.insert_resource(params);

        // Initializing params meta, growth array & texture
        let render_device = render_app.world.get_resource_mut::<RenderDevice>().unwrap();

        let params_buffer = render_device.create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<LeniaGPUParams>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let growth_array_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: None,
            contents: cast_slice(self.lenia_board.get_growth_vector().as_slice()),
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let kernel_image = self.lenia_board.get_kernel_image();

        let image_size = Extent3d {
            width: kernel_image.image.width(),
            height: kernel_image.image.height(),
            ..default()
        };

        let kernel_texture = render_device.create_texture(&TextureDescriptor {
            label: None,
            size: image_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba32Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        // Filling texture with data
        let render_queue = render_app.world.resource::<RenderQueue>();
        render_queue.write_texture(
            ImageCopyTexture {
                texture: &kernel_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &kernel_image.image.as_bytes(),
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(16 * kernel_image.image.width()),
                rows_per_image: NonZeroU32::new(kernel_image.image.height()),
            },
            image_size,
        );

        let kernel_texture_view = kernel_texture.create_view(&TextureViewDescriptor::default());

        render_app.insert_resource(LeniaGPUParamsBuffer::new(params_buffer));
        render_app.insert_resource(LeniaGPUTexture {
            texture_views: vec![kernel_texture_view],
        });
        render_app.insert_resource(LeniaGPUGrowthArrayBuffer::new(growth_array_buffer));
        render_app.add_system(prepare_params.in_set(RenderSet::Prepare));
    }
}

fn prepare_params(
    render_queue: Res<RenderQueue>,
    params_meta: Res<LeniaGPUParamsBuffer>,
    params: Res<LeniaGPUParams>,
    time: Res<Time>,
) {
    let params = params.with_delta_time(time.elapsed_seconds());
    render_queue.write_buffer(&params_meta, 0, cast_slice(&[params]));
}
