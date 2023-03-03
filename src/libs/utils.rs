use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::rc::Rc;
use std::slice;
use super::bounds3::Bounds3;
use super::global::MaterialType;
use super::material::Material;
use super::triangle::Triangle;
use super::vector::Vector3f;


#[link(name = "objloader")]
extern "C" {
    fn create_new_loader() -> *const c_void;
    fn delete_loader(loader: *const c_void);
    fn load_file(loader: *const c_void, file: *const c_char) -> i32;
    fn loaded_meshes(loader: *const c_void, nmesh: *mut i32) -> *const c_void;
    fn mesh_at(meshes: *const c_void, idx: usize) -> *const c_void;
    fn vertex_size_mesh(mesh: *const c_void) -> usize;
    fn mesh_position_at(mesh: *const c_void, idx: usize) -> *const f32;
}

pub unsafe fn load_triangles(filename: &str, m: Rc<Material>) -> (Bounds3, Vec<Triangle>, f32) {
    let loader = create_new_loader();
    let mut triangles = vec![];

    let file: *const c_char = CString::new(filename).unwrap().into_raw();
    load_file(loader, file);
    let mut nmesh: i32 = 0;
    let meshes = loaded_meshes(loader, &mut nmesh as *mut i32);
    assert_eq!(nmesh, 1);
    let mesh = mesh_at(meshes, 0);
    let sz = vertex_size_mesh(mesh);
    let mut bounding_box = Bounds3::empty(Vector3f::zeros());
    let mut j = 0;
    let mut area = 0.0;
    while j < sz {
        let mut face_vertices = [Vector3f::zeros(), Vector3f::zeros(), Vector3f::zeros()];
        for k in 0..3 {
            let vert: Vec<f64> = slice::from_raw_parts(mesh_position_at(mesh, k + j), 3)
                .into_iter().map(|elem| *elem as f64).collect();
            face_vertices[k] = Vector3f::new(vert[0] as f32, vert[1] as f32, vert[2] as f32) * 60.0;
            bounding_box.p_min = Vector3f::min(&bounding_box.p_min, &face_vertices[k]);
            bounding_box.p_max = Vector3f::max(&bounding_box.p_max, &face_vertices[k]);
        }
        j += 3;
        let [v0, v1, v2] = face_vertices;
        let t = Triangle::new(v0, v1, v2, Some(m.clone()));
        area += t.area;
        triangles.push(t);
    }

    delete_loader(loader);
    (bounding_box, triangles, area)
}