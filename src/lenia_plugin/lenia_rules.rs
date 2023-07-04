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
    growth_mapping: Mapping,
}

pub struct KernelShell {
    beta: Vec<f32>,       // kernel peaks
    kernel_core: Mapping, // kernel core K_C : [0, 1] → [0, 1]
}

#[derive(Deref)]
pub struct Mapping(Arc<dyn Fn(f32) -> f32 + Send + Sync>);

#[allow(unused)]
pub enum MappingType {
    GaussianCore { alpha: f32 },
    PolynomialCore { alpha: f32 },
    StepCore,
    GaussianGrowth { mu: f32, sigma: f32 },
    PolynomialGrowth { mu: f32, sigma: f32, alpha: f32 },
    StepGrowth { mu: f32, sigma: f32 },
}

#[allow(dead_code)]
impl Mapping {
    pub fn new(f: Arc<dyn Fn(f32) -> f32 + Send + Sync>) -> Self {
        Self(f)
    }

    pub fn from_type(ty: MappingType) -> Self {
        let func = move |x: f32| match ty {
            MappingType::GaussianCore { alpha } => (alpha - alpha / (4.0 * x * (1.0 - x))).exp(),
            MappingType::PolynomialCore { alpha } => (4.0 * x * (1.0 - x)).powf(alpha),
            MappingType::StepCore => (0.25 <= x && x <= 0.75) as u8 as f32,
            MappingType::GaussianGrowth { mu, sigma } => {
                (-((x - mu).powi(2)) / (2.0 * sigma.powi(2))).exp()
            }
            MappingType::PolynomialGrowth { mu, sigma, alpha } => {
                (((x - mu).abs() <= 3.0 * sigma) as u8 as f32)
                    * (1.0 - (x - mu).powi(2) / (9.0 * sigma.powi(2))).powf(alpha)
            }   
            MappingType::StepGrowth { mu, sigma } => {
                ((x - mu).abs() <= sigma) as u8 as f32
            }
        };
        Self(Arc::new(func))
    }
}

impl From<Arc<dyn Fn(f32) -> f32 + Send + Sync>> for Mapping {
    fn from(value: Arc<dyn Fn(f32) -> f32 + Send + Sync>) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct KernelImage {
    pub image: image::DynamicImage,
    pub area: Vec4,
}

impl KernelImage {
    pub fn new(shell: &KernelShell, radius: u32, zoom: f32) -> Self {
        let center = Vec2::splat(radius as f32);

        let mut buffer = image::Rgba32FImage::new(radius * 2 + 1, radius * 2 + 1);
        let mut area = Vec4::splat(0.0);

        for (x, y, pixel) in buffer.enumerate_pixels_mut() {
            let point = Vec2::new(x as f32, y as f32);
            let dist = center.distance(point) / zoom;
            let normal_dist = dist / radius as f32;

            if normal_dist > 1.0 {
                *pixel = image::Rgba::<f32>([0.0, 0.0, 0.0, 1.0]);
            } else {
                let beta_size = shell.beta.len();
                let kr = normal_dist * beta_size as f32;
                let index = kr.floor() as usize;
                let value;
                if index < beta_size {
                    value = shell.beta[kr.floor() as usize] * (shell.kernel_core)(kr.fract());
                } else {
                    value = 0.0;
                }

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
        r: u32, // cells per kernel radius
        dt: f32,
        growth_resolution: u32,
    ) -> Self {
        if r * 2 + 1 > space_resolution.0 || r * 2 + 1 > space_resolution.1 {
            panic!("diameter is larger than `space_resolution`, kernel will overlap with itself")
        }
        let kernel_image = KernelImage::new(&lenia_rule.kernel_shell, r, 1.0);
        Self {
            lenia_rule,
            space_resolution,
            dx: r,
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
                (self.lenia_rule.growth_mapping)(index as f32 / (self.growth_resolution - 1) as f32)
            })
            .collect()
    }
}

impl LeniaRule {
    pub fn new(kernel_shell: KernelShell, growth_mapping: Mapping) -> Self {
        Self {
            kernel_shell,
            growth_mapping,
        }
    }
}

impl KernelShell {
    pub fn new(beta: Vec<f32>, kernel_core: Mapping) -> Self {
        Self { beta, kernel_core }
    }
}
