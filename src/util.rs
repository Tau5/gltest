use std::ffi::CString;
use nalgebra_glm::{vec3, Vec3};
use openxr::opengl::SessionCreateInfo;
use russimp::Vector3D;
use winit::window::Window;
use glutin::context::PossiblyCurrentContext;
use anyhow::Result;
use glutin::display::GetGlDisplay;
use glutin::prelude::GlDisplay;

pub fn vec3ai_to_glm(vec: Vector3D) -> Vec3 {
    vec3(vec.x, vec.y, vec.z)
}

pub fn session_create_info(
    ctx: &PossiblyCurrentContext,
    #[allow(unused_variables)] window: &Window,
) -> Result<SessionCreateInfo> {
    #[cfg(target_os = "windows")]
    unsafe {
        use glutin::platform::windows::RawHandle;
        use glutin::platform::windows::WindowExtWindows;
        use glutin::platform::ContextTraitExt;

        let hwnd = window.hwnd();
        let h_glrc = match ctx.raw_handle() {
            RawHandle::Wgl(h) => h,
            _ => panic!("EGL not supported here"),
        };

        let h_dc = windows_sys::Win32::Graphics::Gdi::GetDC(hwnd);

        Ok(SessionCreateInfo::Windows {
            h_dc: std::mem::transmute(h_dc),
            h_glrc: std::mem::transmute(h_glrc),
        })
    }

    #[cfg(target_os = "linux")]
    unsafe {
        // See https://gitlab.freedesktop.org/monado/demos/openxr-simple-example/-/blob/master/main.c
        use std::ffi::c_void;
        use glutin_glx_sys::glx::Glx;
        let glx = Glx::load_with(|addr| {
            let symbol = CString::new(addr).unwrap();
            ctx.display().get_proc_address(symbol.as_c_str())
        });
        
        let xlib = glutin_glx_sys::Xlib::open()?;

        let x_display = (xlib.XOpenDisplay)(std::ptr::null());
        let glx_drawable = glx.GetCurrentDrawable();
        let glx_context = glx.GetCurrentContext();

        Ok(SessionCreateInfo::Xlib {
            x_display: std::mem::transmute(x_display),
            visualid: 0,
            glx_fb_config: std::ptr::null::<c_void>() as _,
            glx_drawable,
            glx_context: std::mem::transmute(glx_context),
        })
    }
}