use std::sync::Arc;
use std::time::Instant;
use libs::renderer::Renderer;
use libs::scene::Scene;
use libs::triangle::MeshTriangle;
use libs::vector::Vector3f;
use libs::material::Material;
use libs::global::MaterialType;

mod libs;

fn main() {
    let mut scene = Scene::new(784, 784);

    let mut red = Material::new(MaterialType::Diffuse, Vector3f::zeros(), Vector3f::zeros());
    red.kd = Vector3f::new(0.63, 0.065, 0.05);
    let mut green = Material::new(MaterialType::Diffuse, Vector3f::zeros(), Vector3f::zeros());
    green.kd = Vector3f::new(0.14, 0.45, 0.091);
    let mut white = Material::new(MaterialType::Diffuse, Vector3f::zeros(), Vector3f::zeros());
    white.kd = Vector3f::new(0.725, 0.71, 0.68);
    let mut light = Material::new(MaterialType::Diffuse,
                                  Vector3f::zeros(),
                                  8.0 * Vector3f::new(0.747 + 0.058, 0.747 + 0.258, 0.747) + 15.6 * Vector3f::new(0.740 + 0.287, 0.740 + 0.160, 0.740) + 18.4 * Vector3f::new(0.737 + 0.642, 0.737 + 0.159, 0.737));
    light.kd = Vector3f::same(0.65);

    let (red, white, green, light) = (Arc::new(red), Arc::new(white), Arc::new(green), Arc::new(light));


    let floor = MeshTriangle::from_obj(&"./models/cornellbox/floor.obj", white.clone());
    let shortbox = MeshTriangle::from_obj(&"./models/cornellbox/shortbox.obj", white.clone());
    let tallbox = MeshTriangle::from_obj(&"./models/cornellbox/tallbox.obj", white.clone());
    let left = MeshTriangle::from_obj(&"./models/cornellbox/left.obj", red);
    let right = MeshTriangle::from_obj(&"./models/cornellbox/right.obj", green);
    let light_ = MeshTriangle::from_obj(&"./models/cornellbox/light.obj", light);

    scene.add_obj(Arc::new(floor));
    scene.add_obj(Arc::new(shortbox));
    scene.add_obj(Arc::new(tallbox));
    scene.add_obj(Arc::new(left));
    scene.add_obj(Arc::new(right));
    scene.add_obj(Arc::new(light_));

    scene.build_bvh();

    let start = Instant::now();
    Renderer::render(scene).unwrap();
    println!("Render complete: ");
    println!("Time taken: {:.2} s", start.elapsed().as_secs_f32());
}

// Sequential:
// SPP: 32
// Render complete:
// Time taken: 489.05 s

// Parallel: (16 threads)
// SPP: 32
// Render complete:
// Time taken: 86.44 s