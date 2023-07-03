use bevy::{
    prelude::{Deref, Resource},
    render::render_resource::{Buffer, TextureView},
};
use bytemuck::NoUninit;

#[repr(C)]
#[derive(Resource, Clone, Copy, Debug, NoUninit)]
pub struct LeniaGPUParams {
    pub random_float: f32,
    pub kernel_area: f32,
    pub kernel_resolution: f32,
    pub delta_time: f32,
    pub dt: f32,
    pub growth_resolution: u32,
}

impl LeniaGPUParams {
    pub fn new(random_float: f32, kernel_area: f32, kernel_resolution: f32, dt: f32, growth_resolution: u32) -> Self {
        Self {
            random_float,
            kernel_area,
            kernel_resolution,
            delta_time: 0.0,
            dt,
            growth_resolution
        }
    }

    pub fn with_delta_time(&self, delta_time: f32) -> Self {
        Self {
            delta_time,
            ..*self
        }
    }
}

#[derive(Resource, Deref)]
pub struct LeniaGPUParamsBuffer(Buffer);

impl LeniaGPUParamsBuffer {
    pub fn new(buffer: Buffer) -> Self {
        Self(buffer)
    }
}

#[derive(Resource)]
pub struct LeniaGPUTexture {
    pub texture_views: Vec<TextureView>,
}

#[derive(Resource, Deref)]
pub struct LeniaGPUGrowthArrayBuffer(Buffer);

impl LeniaGPUGrowthArrayBuffer {
    pub fn new(buffer: Buffer) -> Self {
        Self(buffer)
    }
}