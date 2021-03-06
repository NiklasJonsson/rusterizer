use crate::math::*;

pub fn translate_v<CS>(v: Vec3<CS>) -> Mat4<CS>
where
    CS: CoordinateSystem,
{
    translate(v.x(), v.y(), v.z())
}

pub fn translate<CS>(x: f32, y: f32, z: f32) -> Mat4<CS>
where
    CS: CoordinateSystem,
{
    mat4::<CS, CS>(
        1.0, 0.0, 0.0, x, 0.0, 1.0, 0.0, y, 0.0, 0.0, 1.0, z, 0.0, 0.0, 0.0, 1.0,
    )
}

pub fn rotate_x<CS>(rad: f32) -> Mat4<CS>
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

pub fn rotate_y<CS>(rad: f32) -> Mat4<CS>
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

pub fn rotate_z<CS>(rad: f32) -> Mat4<CS>
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

pub fn rotate<CS>(x: f32, y: f32, z: f32) -> Mat4<CS>
where
    CS: CoordinateSystem,
{
    rotate_z(z) * rotate_y(y) * rotate_x(x)
}
