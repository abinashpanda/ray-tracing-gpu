#![cfg_attr(target_arch = "spirv", no_std)]
#![feature(lang_items)]
#![feature(register_attr)]
#![register_attr(spirv)]
#![allow(deprecated)]

use ray_tracing::{main_image, Inputs};
use shared::*;
use spirv_std::glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};
use spirv_std::storage_class::{Input, Output, PushConstant};

pub mod ray_tracing;

pub trait SampleCube: Copy {
    fn sample_cube(self, p: Vec3) -> Vec4;
}

#[derive(Copy, Clone)]
struct ConstantColor {
    color: Vec4,
}

impl SampleCube for ConstantColor {
    fn sample_cube(self, _: Vec3) -> Vec4 {
        self.color
    }
}

#[derive(Copy, Clone)]
struct RgbCube {
    alpha: f32,
    intensity: f32,
}

impl SampleCube for RgbCube {
    fn sample_cube(self, p: Vec3) -> Vec4 {
        (p.abs() * self.intensity).extend(self.alpha)
    }
}

pub fn fs(constants: &ShaderConstants, mut frag_coord: Vec2) -> Vec4 {
    let resolution = vec3(constants.width as f32, constants.height as f32, 0.0);
    let time = constants.time;
    let mut mouse = vec4(
        constants.drag_end_x,
        constants.drag_end_y,
        constants.drag_start_x,
        constants.drag_start_y,
    );
    if mouse != Vec4::zero() {
        mouse.y = resolution.y - mouse.y;
        mouse.w = resolution.y - mouse.w;
    }
    if !constants.mouse_left_pressed {
        mouse.z *= -1.0;
    }
    if !constants.mouse_left_clicked {
        mouse.w *= -1.0;
    }

    frag_coord.x %= resolution.x;
    frag_coord.y = resolution.y - frag_coord.y % resolution.y;

    let mut color = Vec4::zero();

    main_image(
        &mut color,
        frag_coord,
        Inputs {
            time,
            resolution,
            mouse,
        },
    );
    pow(color.truncate(), 2.2).extend(color.w)
}

#[allow(unused_attributes)]
#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] in_frag_coord: Input<Vec4>,
    constants: PushConstant<ShaderConstants>,
    mut output: Output<Vec4>,
) {
    let constants = constants.load();

    let frag_coord = vec2(in_frag_coord.load().x, in_frag_coord.load().y);
    let color = fs(&constants, frag_coord);
    output.store(color);
}

#[allow(unused_attributes)]
#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_idx: Input<i32>,
    #[spirv(position)] mut builtin_pos: Output<Vec4>,
) {
    let vert_idx = vert_idx.load();

    // Create a "full screen triangle" by mapping the vertex index.
    // ported from https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
    let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::one();

    builtin_pos.store(pos.extend(0.0).extend(1.0));
}
