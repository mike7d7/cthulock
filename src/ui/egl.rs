use glutin::{
    api::egl::{context::PossiblyCurrentContext, display::Display, surface::Surface},
    config::ConfigTemplateBuilder,
    context::ContextAttributesBuilder,
    display::GetGlDisplay,
    prelude::*,
    surface::{SurfaceAttributesBuilder, WindowSurface},
};
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use slint::platform::femtovg_renderer::OpenGLInterface;
use std::{
    error::Error,
    ffi::{c_void, CStr},
    num::NonZeroU32, ptr::NonNull,
};
use wayland_client::backend::ObjectId;

pub struct OpenGLContext {
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
}

impl OpenGLContext {
    pub fn new(display_id: ObjectId, surface_id: ObjectId, size: (u32, u32)) -> Self {
        let handle = WaylandDisplayHandle::new(NonNull::<c_void>::new(display_id.as_ptr() as *mut _).expect("error converting to cvoid"));
        let display_handle = RawDisplayHandle::Wayland(handle);

        let config_template = ConfigTemplateBuilder::new().with_alpha_size(8).build();

        let glutin_display =
            unsafe { Display::new(display_handle).expect("Failed to create EGL Display") };

        let config = unsafe { glutin_display.find_configs(config_template) }
            .unwrap()
            .reduce(|config, acc| {
                if config.num_samples() > acc.num_samples() {
                    config
                } else {
                    acc
                }
            })
            .expect("No available configs");

        let context_attributes = ContextAttributesBuilder::new().build(None);

        let not_current = unsafe {
            glutin_display
                .create_context(&config, &context_attributes)
                .expect("Failed to create OpenGL context")
        };

        let handle = WaylandWindowHandle::new(NonNull::<c_void>::new(surface_id.as_ptr() as *mut _).expect("error converting to cvoid"));
        let surface_handle = RawWindowHandle::Wayland(handle);

        let (width, height) = size;

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            surface_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

        let surface = unsafe {
            glutin_display
                .create_window_surface(&config, &attrs)
                .expect("Failed to create OpenGl surface")
        };

        let context = not_current
            .make_current(&surface)
            .expect("Failed to make newly created OpenGL context current");

        Self { context, surface }
    }
}

unsafe impl OpenGLInterface for OpenGLContext {
    fn ensure_current(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        log::debug!("ensuring context is current");
        if !self.context.is_current() {
            log::info!("context not current. Making current");
            self.context.make_current(&self.surface)?;
        }
        Ok(())
    }

    fn swap_buffers(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        log::debug!("swapping buffers");
        self.surface.swap_buffers(&self.context)?;
        Ok(())
    }

    fn resize(
        &self,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.ensure_current()?;
        log::debug!("resizing surface to {width},{height}");
        self.surface.resize(&self.context, width, height);

        Ok(())
    }

    fn get_proc_address(&self, name: &CStr) -> *const c_void {
        self.context.display().get_proc_address(name)
    }
}
