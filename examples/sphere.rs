use std::f32::consts::PI;

use embree4_rs::{
    geometry::SphereGeometry,
    Device, Scene,
};

use anyhow::Result;
use embree4_sys::RTCRay;
use glam::{vec3, Vec3};

fn main() -> Result<()> {
    let config = Some("verbose=1");
    let device = Device::try_new(config)?;
    let scene = Scene::try_new(&device, Default::default())?;

    let center = vec3(0.0, 0.0, 5.0);
    let radius = 1.0;

    let geom = SphereGeometry::try_new(&device, (center.x, center.y, center.z), radius)?;

    scene.attach_geometry(&geom)?;
    let scene = scene.commit()?;

    // Trace rays through each pixel.
    // We use an orthographic camera with the image plane at z=5.
    // We count the hits to estimate pi.

    let width = 4096;
    let height = 4096;

    let cam_dist = center.z;

    let rays = width * height;
    let mut hits = 0;

    let t0 = std::time::Instant::now();
    for x in 0..width {
        for y in 0..height {
            let u = (x as f32 + 0.5) / width as f32;
            let v = (y as f32 + 0.5) / height as f32;

            let target = vec3(u * 2.0 - 1.0, v * 2.0 - 1.0, 5.0);
            let direction = cam_dist * Vec3::Z;
            let origin = target - direction;

            // construct a ray
            let ray = RTCRay {
                org_x: origin.x,
                org_y: origin.y,
                org_z: origin.z,
                dir_x: direction.x,
                dir_y: direction.y,
                dir_z: direction.z,
                ..Default::default()
            };

            let hit = scene.intersect_1(ray)?;

            if hit.is_some() {
                hits += 1
            }
        }
    }
    let elapsed = t0.elapsed();

    let hit_fraction = hits as f32 / rays as f32;
    println!("hit_fraction: {}", hit_fraction);

    let approx_pi = hit_fraction * 4.0;
    println!("   approx_pi: {}", approx_pi);

    let err = (approx_pi - PI).abs();
    let err_percent = err / PI * 100.0;
    println!("         err: {} ({:.5}%)", err, err_percent);

    let rays_per_sec = (rays as f32 / elapsed.as_secs_f32()) as usize;
    println!("rays_per_sec: {}", rays_per_sec);

    Ok(())
}
