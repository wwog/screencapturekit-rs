import CoreGraphics

// MARK: - CGDisplay Bridge

/// 获取当前激活的显示器列表。调用方负责释放返回的 buffer。
@_cdecl("cg_get_active_display_list")
public func cgGetActiveDisplayList(
    _ outPtr: UnsafeMutablePointer<UnsafeMutablePointer<UInt32>?>,
    _ outCount: UnsafeMutablePointer<Int>
) -> Bool {
    var count: UInt32 = 0
    var error = CGGetActiveDisplayList(0, nil, &count)
    guard error == .success else { return false }

    if count == 0 {
        outPtr.pointee = nil
        outCount.pointee = 0
        return true
    }

    let buffer = UnsafeMutablePointer<UInt32>.allocate(capacity: Int(count))
    error = CGGetActiveDisplayList(count, buffer, &count)

    guard error == .success else {
        buffer.deallocate()
        return false
    }

    outPtr.pointee = buffer
    outCount.pointee = Int(count)
    return true
}

@_cdecl("cg_active_display_list_free")
public func cgActiveDisplayListFree(_ ptr: UnsafeMutablePointer<UInt32>?, _ count: Int) {
    ptr?.deinitialize(count: count)
    ptr?.deallocate()
}

/// 获取显示器当前模式（逻辑分辨率 & 像素分辨率 & 刷新率）
@_cdecl("cg_display_copy_current_mode")
public func cgDisplayCopyCurrentMode(
    _ displayID: UInt32,
    _ outWidth: UnsafeMutablePointer<Int32>,
    _ outHeight: UnsafeMutablePointer<Int32>,
    _ outPixelWidth: UnsafeMutablePointer<Int32>,
    _ outPixelHeight: UnsafeMutablePointer<Int32>,
    _ outRefreshRate: UnsafeMutablePointer<Double>
) -> Bool {
    guard let mode = CGDisplayCopyDisplayMode(displayID) else {
        return false
    }

    outWidth.pointee = Int32(mode.width)
    outHeight.pointee = Int32(mode.height)
    outPixelWidth.pointee = Int32(mode.pixelWidth)
    outPixelHeight.pointee = Int32(mode.pixelHeight)
    outRefreshRate.pointee = mode.refreshRate
    return true
}

/// 创建指定显示器的 CGImage（低版本截图备用）
@_cdecl("cg_display_create_image")
public func cgDisplayCreateImage(_ displayID: UInt32) -> OpaquePointer? {
    guard let image = CGDisplayCreateImage(displayID) else {
        return nil
    }

    return OpaquePointer(Unmanaged.passRetained(image).toOpaque())
}

/// 基于可选矩形截取显示器图像（为 0 或负值则截全屏）
@_cdecl("cg_display_create_image_rect")
public func cgDisplayCreateImageRect(
    _ displayID: UInt32,
    _ x: Double,
    _ y: Double,
    _ width: Double,
    _ height: Double
) -> OpaquePointer? {
    if width > 0 && height > 0 {
        let rect = CGRect(x: x, y: y, width: width, height: height)
        guard let image = CGDisplayCreateImage(displayID, rect: rect) else {
            return nil
        }
        return OpaquePointer(Unmanaged.passRetained(image).toOpaque())
    } else {
        // fallback: full display
        guard let image = CGDisplayCreateImage(displayID) else {
            return nil
        }
        return OpaquePointer(Unmanaged.passRetained(image).toOpaque())
    }
}

