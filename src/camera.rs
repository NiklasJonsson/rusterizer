use crate::math::*;

pub struct Camera {
    pos: Point3D<WorldSpace>,
    up: Vec3<WorldSpace>,
    dir: Vec3<WorldSpace>,
}

impl Camera {
    pub fn get_view_matrix(&self) -> Mat4<WorldSpace, CameraSpace> {
        // cam_transform = T * R, view = inverse(cam_transform) = inv(R) * inv(T)

        // Camera looks in negative z of its own space
        // Camera space is left-handed just like world space
        let cam_z = -self.dir;
        let cam_x = cam_z.cross(self.up).normalize();
        let cam_y = cam_x.cross(cam_z).normalize();

        let rotation_inv = mat4(
            cam_x.x(),
            cam_y.x(),
            cam_z.x(),
            0.0,
            cam_x.y(),
            cam_y.y(),
            cam_z.y(),
            0.0,
            cam_x.z(),
            cam_y.z(),
            cam_z.z(),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
        .transpose();

        let vec_to_pos: Vec3<WorldSpace> = self.pos - point::origin();

        let translation_inv = transform::translation_along(-vec_to_pos);
        rotation_inv * translation_inv
    }
}

impl Default for Camera {
    fn default() -> Camera {
        let pos = Point3D::<WorldSpace>::new(0.0, 0.0, -2.0);
        let up = vec3::<WorldSpace>(0.0, 1.0, 0.0).normalize();
        let dir = vec3::<WorldSpace>(0.0, 0.0, 1.0).normalize();

        Camera { pos, up, dir }
    }
}
