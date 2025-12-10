use screencapturekit::{
    cg_display::CGDisplay,
    prelude::{PixelFormat, SCContentFilter, SCShareableContent, SCStreamConfiguration},
    screenshot_manager::{CGImage, SCScreenshotManager},
};
use std::{sync::Arc, time::Instant};

///有几个显示器，就创建几个线程，每个线程负责一个显示器的捕获
fn main() {
    let start_time = Instant::now();
    let content = Arc::new(SCShareableContent::get().unwrap());
    let displays = CGDisplay::active_displays().unwrap();
    let mut threads = Vec::new();
    for display_id in displays {
        let content = content.clone();
        let thread = std::thread::spawn(move || {
            let image = capture_display(display_id, content);
            let path = format!("screenshot_{}.png", display_id);
            image.save_png(&path).unwrap();
        });
        threads.push(thread);
    }
    for thread in threads {
        thread.join().unwrap();
    }
    println!("Time taken: {:?}", start_time.elapsed());
}

/// 获取显示器的原始物理分辨率
///
/// 通过 Core Graphics API 获取显示器的 native mode（最高分辨率）
fn get_native_resolution(display_native_id: u32) -> (usize, usize) {
    let display = CGDisplay::new(display_native_id);

    // 获取当前显示模式
    if let Some(current_mode) = display.display_mode() {
        // 获取像素宽度和高度（物理分辨率）
        let width = current_mode.pixel_width() as usize;
        let height = current_mode.pixel_height() as usize;
        (width, height)
    } else {
        (0, 0)
    }
}

fn capture_display(displayid: u32, content: Arc<SCShareableContent>) -> CGImage {
    let (width, height) = get_native_resolution(displayid);
    println!(
        "displayid: {} native resolution: {}x{}",
        displayid, width, height
    );
    let binding = content.displays();
    let sc_display = binding
        .iter()
        .find(|d| d.display_id() == displayid)
        .unwrap();
    let filter = SCContentFilter::builder()
        .display(sc_display)
        .exclude_windows(&[])
        .build();
    let config = SCStreamConfiguration::new()
        .with_pixel_format(PixelFormat::BGRA)
        .with_width(width as u32)
        .with_shows_cursor(false)
        .with_height(height as u32);

    // > macos 14.0+
    SCScreenshotManager::capture_image(&filter, &config).unwrap()
}
