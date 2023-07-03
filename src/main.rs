//! A compute shader that simulates Conway's Game of Life.
//!
//! Compute shaders use the GPU for computing arbitrary information, that may be independent of what
//! is rendered to the screen.

use std::sync::Arc;

use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
};

mod compute_plugin;
mod lenia_plugin;
use compute_plugin::*;
use lenia_plugin::{
    lenia::{GrowthMapping, GrowthMappingType, KernelImage, KernelShell, LeniaBoard, LeniaRule},
    LeniaRenderPlugin,
};

const SIZE: (u32, u32) = (1280, 720);
const WORKGROUP_SIZE: u32 = 8;

fn main() {
    let gol_kernel_core = |x| {
        if 0.0 <= x && x < 0.25 {
            0.5
        } else if x <= 0.75 {
            1.0
        } else if x <= 1.0 {
            0.0
        } else {
            panic!("{} is not within range [0,1].", x);
        }
    };

    let kernel_shell = KernelShell::new(vec![1.0], Arc::new(gol_kernel_core));
    let kernel_image = KernelImage::new(&kernel_shell, 100, 1.0);

    kernel_image
        .save_image("lenia/assets/kernels/gol_kernel.png")
        .unwrap();

    let lenia_board = LeniaBoard::new(
        LeniaRule::new(
            kernel_shell,
            GrowthMapping::from_type(GrowthMappingType::Rectangular, 0.35, 0.07),
        ),
        SIZE,
        1,
        1.0,
        100,
    );

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // uncomment for unthrottled FPS
                // present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_startup_system(setup)
        .add_plugin(LeniaRenderPlugin::new(lenia_board))
        .add_plugin(LeniaComputePlugin)
        .run();
}

// fn main() {
//     App::new()
//         .insert_resource(ClearColor(Color::BLACK))
//         .add_plugins(DefaultPlugins.set(WindowPlugin {
//             primary_window: Some(Window {
//                 // uncomment for unthrottled FPS
//                 // present_mode: bevy::window::PresentMode::AutoNoVsync,
//                 ..default()
//             }),
//             ..default()
//         }))
//         .add_startup_system(setup)
//         .add_plugin(GameOfLifeComputePlugin)
//         .run();
// }

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(LeniaImage(image));
}
