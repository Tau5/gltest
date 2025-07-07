use std::ffi::CString;
use std::ptr::{null, null_mut};
use nalgebra_glm::{vec3, Vec3};
use openxr::opengl::SessionCreateInfo;
use russimp::Vector3D;
use winit::window::Window;
use glutin::context::PossiblyCurrentContext;
use anyhow::Result;
use glutin::display::GetGlDisplay;
use glutin::prelude::GlDisplay;
use glutin_glx_sys::{glx, Success};

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

        let x_display = glx.GetCurrentDisplay();
        let glx_context = glx.GetCurrentContext();
        let glx_drawable = glx.GetCurrentDrawable();
        let mut config_id = 0;
        assert_eq!(
            glx.QueryContext(
                x_display,
                glx_context,
                glx::FBCONFIG_ID as _,
                &mut config_id
            ),
            Success as i32
        );

        let mut screen = 0;
        assert_eq!(
            glx.QueryContext(x_display, glx_context, glx::SCREEN as _, &mut screen),
            Success as i32
        );

        let attrs = [glx::FBCONFIG_ID, config_id as _, glx::NONE];
        let mut items = 0;
        let cfgs = glx.ChooseFBConfig(x_display, screen, attrs.as_ptr() as _, &mut items);
        let fbconfig = (!cfgs.is_null()).then(|| {
            assert_ne!(items, 0);
            std::slice::from_raw_parts(cfgs, items as usize)[0].cast_mut()
        });
        let visualid = fbconfig
            .map(|cfg| {
                let visual = glx.GetVisualFromFBConfig(x_display, cfg);
                if visual.is_null() {
                    0
                } else {
                    (&raw const (*visual).visualid).read() as u32
                }
            })
            .unwrap_or(0);

        println!("SessionCreateInfo, assemble!");
        Ok(SessionCreateInfo::Xlib {
            x_display: x_display.cast(),
            glx_fb_config: fbconfig.unwrap_or_else(|| {
                std::ptr::null_mut()
            }),
            visualid,
            glx_drawable,
            glx_context: glx_context.cast_mut(),
        })
    }
}