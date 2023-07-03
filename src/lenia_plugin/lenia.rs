// #![allow(unused)]

use std::{sync::Arc, usize};

use crate::lenia_plugin::params;
// use crate::*;

use bevy::{
    math::Vec2,
    prelude::{Deref, Vec4},
};

pub struct LeniaBoard {
    lenia_rule: LeniaRule,
    space_resolution: (u32, u32), // (width, height), the space resolution
    dx: u32, // The site distance, in which the kernel is applied over a site distance of 2
    dt: f32, // timestep
    growth_resolution: u32,
    kernel_image: KernelImage, // Kernel rendered as an image file
}
pub struct LeniaRule {
    kernel_shell: KernelShell,
    growth_mapping: GrowthMapping,
}

pub struct KernelShell {
    beta: Vec<f32>,                                     // kernel peaks
    kernel_core: Arc<dyn Fn(f32) -> f32 + Send + Sync>, // kernel core K_C : [0, 1] → [0, 1]
}

#[derive(Deref)]
pub struct GrowthMapping(Arc<dyn Fn(f32) -> f32 + Send + Sync>);

#[allow(unused)]
pub enum GrowthMappingType {
    Exponential,
    Polynomial { alpha: f32 },
    Rectangular,
}

#[allow(dead_code)]
impl GrowthMapping {
    pub fn new(f: Arc<dyn Fn(f32) -> f32 + Send + Sync>) -> Self {
        Self(f)
    }

    pub fn from_type(ty: GrowthMappingType, mu: f32, sigma: f32) -> Self {
        let func = move |x: f32| match ty {
            GrowthMappingType::Exponential => {
                2.0 * (-((x - mu) * (x - mu)) / (2.0 * sigma * sigma)).exp() - 1.0
            }
            GrowthMappingType::Polynomial { alpha } => {
                let rect = if 0.0 <= x && x < mu - 3.0 * sigma {
                    0.0
                } else if x <= mu + 3.0 * sigma {
                    1.0
                } else if x <= 1.0 {
                    0.0
                } else {
                    panic!("{} is not within range [0,1].", x);
                };

                let poly_num = (1.0 - ((x - mu) * (x - mu)) / (9.0 * sigma * sigma)).powf(alpha);

                2.0 * rect * poly_num - 1.0
            }
            GrowthMappingType::Rectangular => {
                if 0.0 <= x && x < mu - sigma {
                    -1.0
                } else if x <= mu + sigma {
                    1.0
                } else if x <= 1.0 {
                    -1.0
                } else {
                    panic!("{} is not within range [0,1].", x);
                }
            }
        };
        Self(Arc::new(func))
    }
}

#[derive(Clone)]
pub struct KernelImage {
    pub image: image::DynamicImage,
    pub area: Vec4,
}

impl KernelImage {
    pub fn new(shell: &KernelShell, resolution: u32, zoom: f32) -> Self {
        let center = Vec2::splat((resolution as f32 - 1.0) / 2.0);

        let mut buffer = image::Rgba32FImage::new(resolution, resolution);
        let mut area = Vec4::splat(0.0);

        for (x, y, pixel) in buffer.enumerate_pixels_mut() {
            let point = Vec2::new(x as f32, y as f32);
            let dist = center.distance(point) / zoom;
            let normal_dist = dist / resolution as f32;

            if normal_dist > 1.0 {
                *pixel = image::Rgba::<f32>([0.0, 0.0, 0.0, 1.0]);
            } else {
                let beta_size = shell.beta.len() as f32;
                let kr = normal_dist * beta_size;
                let value = shell.beta[kr.floor() as usize] * (shell.kernel_core)(kr.fract());

                *pixel = image::Rgba::<f32>([value, value, value, 1.0]);
                area += Vec4::splat(value);
            }
        }

        Self {
            image: image::DynamicImage::ImageRgba32F(buffer),
            area,
        }
    }

    /// Saves kernel as an image file, not useful for calculation purposes.
    pub fn save_image(&self, path: &str) -> image::ImageResult<()> {
        self.image.clone().into_rgba16().save(path)
    }
}

impl LeniaBoard {
    pub fn new(
        lenia_rule: LeniaRule,
        space_resolution: (u32, u32),
        dx: u32,
        dt: f32,
        growth_resolution: u32,
    ) -> Self {
        if dx * 2 + 1 > space_resolution.0 || dx * 2 + 1 > space_resolution.1 {
            panic!("`dx` is larger than `space_resolution`, kernel will overlap with itself")
        }
        let kernel_image = KernelImage::new(&lenia_rule.kernel_shell, dx * 2 + 1, 1.0);
        Self {
            lenia_rule,
            space_resolution,
            dx,
            dt,
            growth_resolution,
            kernel_image,
        }
    }

    pub fn generate_params(&self) -> params::LeniaGPUParams {
        params::LeniaGPUParams::new(
            rand::random::<f32>(),
            self.kernel_image.area.x,
            (self.dx * 2 + 1) as f32,
            self.dt,
            self.growth_resolution,
        )
    }

    pub fn get_kernel_image(&self) -> KernelImage {
        self.kernel_image.clone()
    }

    pub fn get_growth_vector(&self) -> Vec<f32> {
        (0..self.growth_resolution)
            .map(|index| {
                (self.lenia_rule.growth_mapping)(
                    index as f32 / (self.growth_resolution - 1) as f32,
                )
            })
            .collect()
    }
}

impl LeniaRule {
    pub fn new(kernel_shell: KernelShell, growth_mapping: GrowthMapping) -> Self {
        Self {
            kernel_shell,
            growth_mapping,
        }
    }
}

impl KernelShell {
    pub fn new(beta: Vec<f32>, kernel_core: Arc<dyn Fn(f32) -> f32 + Send + Sync>) -> Self {
        Self { beta, kernel_core }
    }
}
