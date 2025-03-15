use anyhow::{bail, Result};

use crate::{device_error_or, Device};

use super::Geometry;

pub struct SphereGeometry {
    handle: embree4_sys::RTCGeometry,
}

impl SphereGeometry {
    /// Constructs a new `SphereGeometry` instance for a single sphere defined by position and radius.
    ///
    /// # Example
    /// ```
    /// use embree4_rs::{*, geometry::*};
    /// use embree4_sys::*;
    ///
    /// let device = Device::try_new(None).unwrap();
    /// let geometry = SphereGeometry::try_new(&device, (0.0, 0.0, 0.0), 1.0).unwrap();
    /// let scene = Scene::try_new(&device, SceneOptions::default()).unwrap();
    /// scene.attach_geometry(&geometry);
    /// ```
    pub fn try_new(device: &Device, position: (f32, f32, f32), radius: f32) -> Result<Self> {
        // Create a new geometry with RTC_GEOMETRY_TYPE_SPHERE_POINT
        let geometry = unsafe {
            embree4_sys::rtcNewGeometry(device.handle, embree4_sys::RTCGeometryType::SPHERE_POINT)
        };
        if geometry.is_null() {
            bail!("Failed to create sphere geometry: {:?}", device.error());
        }

        // Allocate vertex buffer for a single sphere (x, y, z, radius)
        let vertex_buf_ptr = unsafe {
            embree4_sys::rtcSetNewGeometryBuffer(
                geometry,
                embree4_sys::RTCBufferType::VERTEX,
                0,
                embree4_sys::RTCFormat::FLOAT4,         // (x, y, z, r)
                4 * std::mem::size_of::<f32>(), // 16 bytes
                1,                        // Just one sphere
            )
        };
        if vertex_buf_ptr.is_null() {
            bail!("Failed to create sphere vertex buffer: {:?}", device.error());
        }
        device_error_or(device, (), "Failed to create sphere vertex buffer")?;

        // Convert raw pointer to a mutable slice for filling data
        let vertex_buf = unsafe { std::slice::from_raw_parts_mut(vertex_buf_ptr as *mut f32, 4) };

        // Set the sphere data
        vertex_buf[0] = position.0; // x
        vertex_buf[1] = position.1; // y
        vertex_buf[2] = position.2; // z
        vertex_buf[3] = radius;     // radius

        // Commit the geometry
        unsafe {
            embree4_sys::rtcCommitGeometry(geometry);
        }
        device_error_or(device, (), "Failed to commit sphere geometry")?;

        Ok(Self { handle: geometry })
    }
}

impl Drop for SphereGeometry {
    fn drop(&mut self) {
        unsafe {
            embree4_sys::rtcReleaseGeometry(self.handle);
        }
    }
}

impl Geometry for SphereGeometry {
    fn geometry(&self) -> embree4_sys::RTCGeometry {
        self.handle
    }
}
