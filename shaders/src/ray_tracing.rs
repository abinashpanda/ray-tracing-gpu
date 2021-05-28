use spirv_std::glam::{Vec2, Vec3, Vec4};

// Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// we tie #[no_std] above to the same condition, so it's fine.
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

pub struct Inputs {
    pub resolution: Vec3,
    pub time: f32,
    pub mouse: Vec4,
}

pub fn main_image(frag_color: &mut Vec4, _frag_coord: Vec2, _inputs: Inputs) {
    *frag_color = Vec4::new(1.0, 0.0, 0.0, 1.0);
}
