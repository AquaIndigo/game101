use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::slice;
use nalgebra::{Matrix3, Matrix4, Vector3, Vector4};
use opencv::core::{Mat, MatTraitConst};
use opencv::imgproc::{COLOR_RGB2BGR, cvt_color};
use crate::shader::{FragmentShaderPayload, VertexShaderPayload};
use crate::texture::Texture;
use crate::triangle::Triangle;

type V3f = Vector3<f64>;
type M4f = Matrix4<f64>;

pub(crate) fn get_view_matrix(eye_pos: V3f) -> M4f {
    let mut view: M4f = Matrix4::identity();
    view[(0, 3)] = -eye_pos[0];
    view[(1, 3)] = -eye_pos[1];
    view[(2, 3)] = -eye_pos[2];

    view
}

pub(crate) fn get_model_matrix(rotation_angle: f64) -> M4f {
    let mut model: M4f = Matrix4::identity();
    let rad = rotation_angle.to_radians();
    model[(0, 0)] = rad.cos();
    model[(2, 2)] = model[(0, 0)];
    model[(0, 2)] = rad.sin();
    model[(2, 0)] = -model[(0, 2)];
    let mut scale: M4f = Matrix4::identity();
    scale[(0, 0)] = 2.5;
    scale[(1, 1)] = 2.5;
    scale[(2, 2)] = 2.5;
    model * scale
}

pub(crate) fn get_projection_matrix(eye_fov: f64, aspect_ratio: f64, z_near: f64, z_far: f64) -> M4f {
    let mut persp2ortho: M4f = Matrix4::zeros();
    let (n, f) = (z_near, z_far);
    let (a, b) = (n + f, -n * f);
    persp2ortho[(0, 0)] = n;
    persp2ortho[(1, 1)] = n;
    persp2ortho[(3, 2)] = 1.0;
    persp2ortho[(2, 2)] = a;
    persp2ortho[(2, 3)] = b;
    let mut scale: M4f = Matrix4::identity();
    let mut tran: M4f = Matrix4::identity();
    let t = -(eye_fov / 2.0).to_radians().tan() * n.abs();
    let r = aspect_ratio * t;
    let (l, b) = (-r, -t);
    scale[(0, 0)] = 2.0 / (r - l);
    scale[(1, 1)] = 2.0 / (t - b);
    scale[(2, 2)] = 2.0 / (n - f);
    tran[(0, 3)] = -(r + l) / 2.0;
    tran[(1, 3)] = -(t + b) / 2.0;
    tran[(2, 3)] = -(n + f) / 2.0;

    scale * tran * persp2ortho
}


pub(crate) fn frame_buffer2cv_mat(frame_buffer: &Vec<V3f>) -> Mat {
    let mut image = unsafe {
        Mat::new_rows_cols_with_data(
            700, 700,
            opencv::core::CV_64FC3,
            frame_buffer.as_ptr() as *mut c_void,
            opencv::core::Mat_AUTO_STEP,
        ).unwrap()
    };
    let mut img = Mat::copy(&image).unwrap();
    image.convert_to(&mut img, opencv::core::CV_8UC3, 1.0, 1.0).expect("panic message");
    cvt_color(&img, &mut image, COLOR_RGB2BGR, 0).unwrap();
    image
}

#[link(name = "objloader")]
extern "C" {
    fn create_new_loader() -> *const c_void;
    fn delete_loader(loader: *const c_void);
    fn load_file(loader: *const c_void, file: *const c_char) -> i32;
    fn loaded_meshes(loader: *const c_void, nmesh: *mut i32) -> *const c_void;
    fn mesh_at(meshes: *const c_void, idx: usize) -> *const c_void;
    fn vertex_size_mesh(mesh: *const c_void) -> usize;
    fn mesh_position_at(mesh: *const c_void, idx: usize) -> *const f32;
    fn mesh_normal_at(mesh: *const c_void, idx: usize) -> *const f32;
    fn mesh_texture_at(mesh: *const c_void, idx: usize) -> *const f32;
}

pub unsafe fn load_triangles() -> Vec<Triangle> {
    let mut triangles = vec![];
    let loader = create_new_loader();

    let file: *const c_char = CString::new("./models/spot/spot_triangulated_good.obj").unwrap().into_raw();
    load_file(loader, file);

    let mut nmesh: i32 = 0;
    let meshes = loaded_meshes(loader, &mut nmesh as *mut i32);
    for i in 0..nmesh as usize {
        let mesh = mesh_at(meshes, i);
        let sz = vertex_size_mesh(mesh);
        let mut j = 0;
        while j < sz {
            let mut t = Triangle::default();
            for k in 0..3 {
                let res: Vec<f64> = slice::from_raw_parts(mesh_position_at(mesh, k + j), 3)
                    .into_iter().map(|elem| *elem as f64).collect();
                t.set_vertex(k, Vector4::new(res[0], res[1], res[2], 1.0));

                let res: Vec<f64> = slice::from_raw_parts(mesh_normal_at(mesh, k + j), 3)
                    .into_iter().map(|elem| *elem as f64).collect();
                t.set_normal(k, Vector3::new(res[0], res[1], res[2]));
                let res: Vec<f64> = slice::from_raw_parts(mesh_texture_at(mesh, k + j), 2)
                    .into_iter().map(|elem| *elem as f64).collect();
                t.set_tex_coord(k, res[0], res[1]);
            }
            j += 3;

            triangles.push(t);
        }
    }

    delete_loader(loader);
    triangles
}

pub fn choose_shader_texture(method: &str,
                             obj_path: &str) -> (fn(&FragmentShaderPayload) -> Vector3<f64>, Option<Texture>) {
    let mut active_shader: fn(&FragmentShaderPayload) -> Vector3<f64> = phong_fragment_shader;
    let mut tex = None;
    if method == "normal" {
        println!("Rasterizing using the normal shader");
        active_shader = normal_fragment_shader;
    } else if method == "texture" {
        println!("Rasterizing using the normal shader");
        active_shader = texture_fragment_shader;
        tex = Some(Texture::new(&(obj_path.to_owned() + "spot_texture.png")));
    } else if method == "phong" {
        println!("Rasterizing using the phong shader");
        active_shader = phong_fragment_shader;
    } else if method == "bump" {
        println!("Rasterizing using the bump shader");
        active_shader = bump_fragment_shader;
    } else if method == "displacement" {
        println!("Rasterizing using the displacement shader");
        active_shader = displacement_fragment_shader;
    }
    (active_shader, tex)
}

pub fn vertex_shader(payload: &VertexShaderPayload) -> V3f {
    payload.position
}

#[derive(Default)]
struct Light {
    pub position: V3f,
    pub intensity: V3f,
}

fn cal_res_color(res: &mut V3f,
                 [ka, kd, ks]: [&V3f; 3],
                 point: &V3f,
                 normal: &V3f,
                 p: f64,
) {
    let l1 = Light {
        position: Vector3::new(20.0, 20.0, 20.0),
        intensity: Vector3::new(500.0, 500.0, 500.0),
    };
    let l2 = Light {
        position: Vector3::new(-20.0, 20.0, 0.0),
        intensity: Vector3::new(500.0, 500.0, 500.0),
    };
    let lights = vec![l1, l2];
    let eye_pos = Vector3::new(0.0, 0.0, 10.0);
    let amb_light_intensity = Vector3::new(10.0, 10.0, 10.0);


    for light in lights {
        let la = ka.component_mul(&amb_light_intensity);

        let v = (eye_pos - point).normalize();
        let l = (light.position - point).normalize();
        let h = (v + l).normalize();
        let rsq = (light.position - point).dot(&(light.position - point));

        let df = normal.normalize().dot(&l);

        let ld = kd.component_mul(&(light.intensity / rsq)) * df.max(0.0);
        let ls = ks.component_mul(&(light.intensity / rsq)) * h.dot(&normal).powf(p);
        *res += la + ld + ls;
    }
}

pub fn normal_fragment_shader(payload: &FragmentShaderPayload) -> V3f {
    let result_color =
        (payload.normal.xyz().normalize() + Vector3::new(1.0, 1.0, 1.0)) / 2.0;
    result_color * 255.0
}

pub fn phong_fragment_shader(payload: &FragmentShaderPayload) -> V3f {
    let ka = Vector3::new(0.005, 0.005, 0.005);
    let kd = payload.color;
    let ks = Vector3::new(0.7937, 0.7937, 0.7937);

    let normal = payload.normal;
    let point = payload.view_pos;
    let p = 150.0;

    let mut result_color = Vector3::zeros();
    cal_res_color(&mut result_color, [&ka, &kd, &ks], &point, &normal, p);

    result_color * 255.0
}

pub fn texture_fragment_shader(payload: &FragmentShaderPayload) -> V3f {
    let ka = Vector3::new(0.005, 0.005, 0.005);
    let kd = match &payload.texture {
        None => Vector3::new(0.0, 0.0, 0.0),
        Some(texture) => texture.get_color(payload.tex_coords.x, payload.tex_coords.y) / 255.0
    };
    let ks = Vector3::new(0.7937, 0.7937, 0.7937);

    let normal = payload.normal;
    let point = payload.view_pos;
    let p = 150.0;

    let mut result_color = Vector3::zeros();
    cal_res_color(&mut result_color, [&ka, &kd, &ks], &point, &normal, p);
    result_color * 255.0
}

pub fn bump_fragment_shader(payload: &FragmentShaderPayload) -> V3f {
    let (kh, kn) = (0.2, 0.1);
    let normal = payload.normal;
    let [[x, y, z]] = normal.data.0;
    let t = Vector3::new(x * y, y * z, z * x) / (x * x + z * z).sqrt();
    let b = normal.cross(&t);
    let tbn = Matrix3::new(
        t.x, b.x, normal.x,
        t.y, b.y, normal.y,
        t.z, b.z, normal.z);
    let [[u, v]] = payload.tex_coords.xy().data.0;
    let texture = payload.texture.as_ref().unwrap();
    let (w, h) = (texture.width, texture.height);

    let du = kh * kn * (texture.get_color(u + 1.0 / w as f64, v).norm()
        - texture.get_color(u, v).norm());
    let dv = kh * kn * (texture.get_color(u, 1.0 / h as f64 + v).norm()
        - texture.get_color(u, v).norm());
    let ln = Vector3::new(-du, -dv, 1.0);
    let res = (tbn * ln).normalize();
    res * 255.0
}

pub fn displacement_fragment_shader(payload: &FragmentShaderPayload) -> V3f {
    let ka = Vector3::new(0.005, 0.005, 0.005);
    let kd = payload.color;
    let ks = Vector3::new(0.7937, 0.7937, 0.7937);

    let (kh, kn) = (0.2, 0.1);
    let mut normal = payload.normal;
    let mut point = payload.view_pos;
    let p = 150.0;

    let [[x, y, z]] = normal.data.0;
    let t = Vector3::new(x * y, y * z, z * x) / (x * x + z * z).sqrt();
    let b = normal.cross(&t);
    let tbn = Matrix3::new(
        t.x, b.x, normal.x,
        t.y, b.y, normal.y,
        t.z, b.z, normal.z);
    let [[u, v]] = payload.tex_coords.xy().data.0;
    let texture = payload.texture.as_ref().unwrap();
    let (w, h) = (texture.width, texture.height);

    let du = kh * kn * (texture.get_color(u + 1.0 / w as f64, v).norm()
        - texture.get_color(u, v).norm());
    let dv = kh * kn * (texture.get_color(u, 1.0 / h as f64 + v).norm()
        - texture.get_color(u, v).norm());
    let ln = Vector3::new(-du, -dv, 1.0);

    point += kn * normal * texture.get_color(u, v).norm();
    normal = (tbn * ln).normalize();
    let mut res = Vector3::new(0.0, 0.0, 0.0);

    cal_res_color(&mut res, [&ka, &kd, &ks], &point, &normal, p);

    res * 255.0
}
