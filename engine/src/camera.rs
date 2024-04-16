use std::f32::consts::PI;

use cgmath::{
    dot, num_traits::signum, vec4, Decomposed, Deg, InnerSpace, Matrix4, Point3, Quaternion, Rad,
    Rotation3, Vector3,
};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

use crate::utils::Instance;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_PI: f32 = PI - 0.0001;
#[derive(Debug)]
pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub radius: f32,
    pub min_radius: f32,
    pub max_radius: f32,
    pub last_mouse_pos: Option<(f32, f32)>,
    pub inertia: Option<f32>,
    pub inertia_decay: f32,
    pub view_port: Option<(f32, f32)>,
}

impl Camera {
    pub fn new(
        eye: Point3<f32>,
        target: Point3<f32>,
        up: Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
        radius: f32,
        min_radius: f32,
        max_radius: f32,
    ) -> Self {
        Camera {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            radius,
            min_radius,
            max_radius,
            last_mouse_pos: None,
            inertia: Some(0.0), // Set initial inertia to zero
            inertia_decay: 0.9, // Set decay factor for inertia
            view_port: None,
        }
    }
    pub fn set_view_port(&mut self, size: Option<(f32, f32)>) {
        self.view_port = size;
    }
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // 2.
        let proj = cgmath::perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        // 3.
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
    pub fn get_view_dir(&self) -> Vector3<f32> {
        // Calculate the view direction as a unit vector pointing from the eye position to the target position
        (self.target - self.eye).normalize()
    }
    pub fn get_look_at(&self) -> Matrix4<f32> {
        // Calculate the view matrix (look-at matrix) using the eye position, target position, and up vector
        Matrix4::look_to_rh(Point3::from(self.eye), self.get_view_dir(), self.up)
    }
    pub fn get_right_vector(&self) -> Vector3<f32> {
        // Calculate the right vector by taking the cross product of the up vector and the direction vector
        self.up.cross(self.get_view_dir()).normalize()
    }

    pub fn set_camera_view(&mut self, eye: Point3<f32>, target: Point3<f32>, up: Vector3<f32>) {
        self.eye = eye;
        self.target = target;
        self.up = up;
    }

    pub fn update_mouse_position(&mut self, new_mouse_pos: (f32, f32)) {
        if let Some((last_x, last_y)) = self.last_mouse_pos {
            let (width, height) = self.view_port.unwrap_or((1.0, 1.0));

            let mut position = vec4(self.eye.x, self.eye.y, self.eye.z, 1.0);
            let pivot = vec4(self.target.x, self.target.y, self.target.z, 1.0 as f32);

            let delta_angle_x = 2.0 * PI as f32 / width;
            let delta_angle_y = PI as f32 / height;

            let delta_x = (last_x - new_mouse_pos.0) * delta_angle_x;
            let delta_y = (new_mouse_pos.1 - last_y) * delta_angle_y;

            let rotation_matrix_x = Matrix4::from_axis_angle(self.up, Rad(delta_x));
            position = (rotation_matrix_x * (position - pivot)) + pivot;
            
            let rotation_matrix_y = Matrix4::from_axis_angle(self.get_right_vector(), Rad(delta_y));
            let final_pos = (rotation_matrix_y * (position - pivot)) + pivot;
            
                self.set_camera_view(
                    Point3::new(final_pos.x, final_pos.y, final_pos.z),
                    self.target,
                    self.up,
                );

        }

        self.last_mouse_pos = Some(new_mouse_pos);
    }

    pub fn zoom(&mut self, delta_radius: f32) {
        self.radius -= delta_radius;
        self.radius = self.radius.clamp(self.min_radius, self.max_radius);

        let direction = (self.eye - self.target).normalize();
        self.eye = self.target + direction * self.radius;
    }

    pub fn set_last_mouse_position(&mut self, last_mouse_pos: Option<(f32, f32)>) {
        self.last_mouse_pos = last_mouse_pos;
    }

    pub fn update(&mut self) {
        // Update inertia

        if let Some(inertia) = self.inertia {
            if inertia > 0.001 {
                // Minimum inertia threshold
                self.eye = self.target + (self.eye - self.target) * inertia;
                self.inertia = Some(inertia * self.inertia_decay);
            } else {
                self.inertia = None; // Inertia decayed to zero, stop updating
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    // convert the Matrix4 into a 4x4 f32 array
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_position = camera.eye.to_homogeneous().into();
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    pub sensitivity: f32,
    pub is_left_mouse_pressed: bool,
}

impl CameraController {
    pub fn new(sensitivity: f32) -> Self {
        Self {
            sensitivity,
            is_left_mouse_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent, camera: &mut Camera) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                if self.is_left_mouse_pressed {
                    let delta_x = position.x as f64;
                    let delta_y = position.y as f64;
                    camera.update_mouse_position((delta_x as f32, delta_y as f32));
                }
                camera.set_last_mouse_position(Some((*position).into()));

                true
            }
            WindowEvent::Resized(physical_size) => {
                camera.set_view_port(Some((*physical_size).into()));
                camera.aspect = (physical_size.width as f32 / physical_size.height as f32);
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        // Adjust the camera's distance based on the mouse wheel input
                        camera.zoom(*y as f32 * self.sensitivity);
                    }
                    MouseScrollDelta::PixelDelta(phys_pos) => {
                        // Handle pixel delta if needed
                        camera.zoom(phys_pos.y as f32 * 0.01);
                    }
                }
                true
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    self.is_left_mouse_pressed = *state == ElementState::Pressed;
                }
                true
            }
            _ => false,
        }
    }
}
