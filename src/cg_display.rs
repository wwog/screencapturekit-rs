use crate::error::SCError;
use crate::screenshot_manager::CGImage;

/// CoreGraphics 显示模式（当前模式）
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DisplayMode {
    /// 逻辑分辨率（HiDPI 缩放后的 points）
    logical_width: i32,
    logical_height: i32,
    /// 物理像素分辨率（native / backing pixels）
    pixel_width: i32,
    pixel_height: i32,
    refresh_rate: f64,
}

impl DisplayMode {
    #[must_use]
    pub const fn pixel_width(&self) -> i32 {
        self.pixel_width
    }

    #[must_use]
    pub const fn pixel_height(&self) -> i32 {
        self.pixel_height
    }

    #[must_use]
    pub const fn logical_width(&self) -> i32 {
        self.logical_width
    }

    #[must_use]
    pub const fn logical_height(&self) -> i32 {
        self.logical_height
    }

    #[must_use]
    pub const fn refresh_rate(&self) -> f64 {
        self.refresh_rate
    }
}

/// CoreGraphics 显示设备（基于 CGDirectDisplayID）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CGDisplay {
    id: u32,
}

impl CGDisplay {
    /// 从原始 display id 构建
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self { id }
    }

    /// 返回 display id
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id
    }

    /// 获取当前激活的显示器 ID 列表
    pub fn active_displays() -> Result<Vec<u32>, SCError> {
        let mut ptr: *mut u32 = std::ptr::null_mut();
        let mut count: usize = 0;
        let ok = unsafe { crate::ffi::cg_get_active_display_list(&mut ptr, &mut count) };
        if !ok {
            return Err(SCError::internal_error("Failed to get active display list"));
        }

        let ids = if count > 0 && !ptr.is_null() {
            unsafe { std::slice::from_raw_parts(ptr, count).to_vec() }
        } else {
            Vec::new()
        };

        unsafe {
            crate::ffi::cg_active_display_list_free(ptr, count);
        }

        Ok(ids)
    }

    /// 获取当前显示模式
    pub fn display_mode(&self) -> Option<DisplayMode> {
        let mut logical_width: i32 = 0;
        let mut logical_height: i32 = 0;
        let mut pixel_width: i32 = 0;
        let mut pixel_height: i32 = 0;
        let mut refresh_rate: f64 = 0.0;

        let ok = unsafe {
            crate::ffi::cg_display_copy_current_mode(
                self.id,
                &mut logical_width,
                &mut logical_height,
                &mut pixel_width,
                &mut pixel_height,
                &mut refresh_rate,
            )
        };

        if ok {
            Some(DisplayMode {
                logical_width,
                logical_height,
                pixel_width,
                pixel_height,
                refresh_rate,
            })
        } else {
            None
        }
    }

    /// 创建当前显示器的 CGImage（适用于低版本截图回退）
    pub fn create_image(&self) -> Option<CGImage> {
        let image_ptr = unsafe { crate::ffi::cg_display_create_image(self.id) };
        if image_ptr.is_null() {
            None
        } else {
            // Swift 侧已 retain，Rust Drop 会 release
            Some(CGImage::from_ptr(image_ptr))
        }
    }

    /// 按指定矩形创建 CGImage；若 `rect` 为 None 或宽高<=0，则截取全屏
    pub fn create_image_in_rect(&self, rect: crate::cg::CGRect) -> Option<CGImage> {
        let image_ptr = unsafe {
            crate::ffi::cg_display_create_image_rect(
                self.id,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
            )
        };
        if image_ptr.is_null() {
            None
        } else {
            // Swift 侧已 retain，Rust Drop 会 release
            Some(CGImage::from_ptr(image_ptr))
        }
    }
}
