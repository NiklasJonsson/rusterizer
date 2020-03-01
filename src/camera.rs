use crate::math::*;

pub struct Camera {
    pos: Point4D<WorldSpace>,
    up: Vec4<WorldSpace>,
    dir: Vec4<WorldSpace>,
}

impl Camera {
    pub fn get_view_matrix(&self) -> Mat4<WorldSpace, CameraSpace> {
        // cam_transform = T * R, view = inverse(cam_transform) = inv(R) * inv(T)

        let cam_forward = -self.dir;
        let cam_right = self.up.cross(cam_forward).normalize();
        let cam_up = cam_forward.cross(cam_right).normalize();

        let rotation_inv = mat4(
            cam_right.x(),
            cam_right.y(),
            cam_right.z(),
            0.0,
            cam_up.x(),
            cam_up.y(),
            cam_up.z(),
            0.0,
            cam_forward.x(),
            cam_forward.y(),
            cam_forward.z(),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        );

        let vec_to_pos: Vec4<WorldSpace> = self.pos - point::origin();

        let translation_inv = transform::translation_along(-vec_to_pos);
        rotation_inv * translation_inv
    }
}

impl Default for Camera {
    fn default() -> Camera {
        let pos = Point4D::<WorldSpace>::new(0.0, 0.0, 2.0, 1.0);
        let up = vec4::<WorldSpace>(0.0, 1.0, 0.0, 0.0).normalize();
        let dir = vec4::<WorldSpace>(0.0, 0.0, -1.0, 0.0).normalize();

        Camera { pos, up, dir }
    }
}
