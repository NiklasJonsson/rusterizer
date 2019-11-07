use crate::math::*;

pub fn translation_along<CS>(v: Vec4<CS>) -> Mat4<CS, CS>
where
    CS: CoordinateSystem,
{
    mat4::<CS, CS>(
        1.0,
        0.0,
        0.0,
        v.x(),
        0.0,
        1.0,
        0.0,
        v.y(),
        0.0,
        0.0,
        1.0,
        v.z(),
        0.0,
        0.0,
        0.0,
        1.0,
    )
}
