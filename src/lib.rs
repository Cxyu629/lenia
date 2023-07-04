#![allow(unused_imports)]
pub mod compute_plugin;
pub mod lenia_plugin;
pub use compute_plugin::*;
pub use lenia_plugin::{lenia_rules::*, LeniaRenderPlugin};
pub use std::sync::Arc;

pub use bevy::{
    prelude::*,
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
