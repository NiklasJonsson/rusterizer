use crate::math::*;

pub fn translation_along<CS>(v: Vec3<CS>) -> Mat4<CS, CS>
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

pub fn rotate_x<CS>(rad: f32) -> Mat4<CS, CS>
where
    CS: CoordinateSystem,
{
    mat4::<CS, CS>(
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        rad.cos(),
        -rad.sin(),
        0.0,
        0.0,
        rad.sin(),
        rad.cos(),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

pub fn rotate_y<CS>(rad: f32) -> Mat4<CS, CS>
where
    CS: CoordinateSystem,
{
    mat4::<CS, CS>(
        rad.cos(),
        0.0,
        rad.sin(),
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        -rad.sin(),
        0.0,
        rad.cos(),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

pub fn rotate_z<CS>(rad: f32) -> Mat4<CS, CS>
where
    CS: CoordinateSystem,
{
    mat4::<CS, CS>(
        rad.cos(),
        -rad.sin(),
        0.0,
        0.0,
        rad.sin(),
        rad.cos(),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}
