use screencapturekit::{
    prelude::{CGDisplay, PixelFormat, SCContentFilter, SCShareableContent, SCStreamConfiguration},
    screenshot_manager::capture_sample_buffer_with_stream,
};
use std::thread;
use std::time::Instant;

/// 基于 SCStream 的单帧截图示例（支持 macOS 12.3+），返回 CMSampleBuffer
///
/// - 使用 content filter 锁定指定 display
/// - 使用 stream configuration 指定输出分辨率（优先选用 display 的物理像素）
/// - 调用 `capture_sample_buffer_with_stream` 返回 CMSampleBuffer
/// - 可以从 CMSampleBuffer 中提取 CVPixelBuffer 和访问时间戳等元数据
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let start_time = Instant::now();
    let content = SCShareableContent::get()?;
    let displays = content.displays();

    if displays.is_empty() {
        eprintln!("No displays found");
        return Ok(());
    }

    let mut handles = Vec::new();

    for display in displays {
        handles.push(thread::spawn(
            move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let mode = CGDisplay::new(display.display_id()).display_mode();
                let (width, height) = if let Some(m) = mode {
                    // 优先使用物理像素分辨率
                    (m.pixel_width() as u32, m.pixel_height() as u32)
                } else {
                    // 回退到 shareable content 的尺寸
                    (display.width(), display.height())
                };

                let filter = SCContentFilter::builder()
                    .display(&display)
                    .exclude_windows(&[])
                    .build();

                let config = SCStreamConfiguration::new()
                    .with_width(width)
                    .with_height(height)
                    .with_pixel_format(PixelFormat::BGRA)
                    .with_shows_cursor(true);

                let sample_buffer = capture_sample_buffer_with_stream(&filter, &config)?;

                // 从 CMSampleBuffer 中提取 CVPixelBuffer
                if let Some(pixel_buffer) = sample_buffer.image_buffer() {
                    println!(
                        "Captured display {} at {}x{} -> Pixel buffer: {}x{}",
                        display.display_id(),
                        width,
                        height,
                        pixel_buffer.width(),
                        pixel_buffer.height()
                    );
                } else {
                    println!(
                        "Captured display {} at {}x{} -> No pixel buffer",
                        display.display_id(),
                        width,
                        height
                    );
                }

                // 访问时间戳等元数据
                let pts = sample_buffer.presentation_timestamp();
                println!(
                    "Display {} presentation time: {} / {}",
                    display.display_id(),
                    pts.value,
                    pts.timescale
                );

                // 检查帧状态
                if let Some(status) = sample_buffer.frame_status() {
                    println!("Display {} frame status: {:?}", display.display_id(), status);
                }

                Ok(())
            },
        ));
    }

    for handle in handles {
        handle.join().unwrap()?;
    }

    println!("Time taken: {:?}", start_time.elapsed());
    Ok(())
}
