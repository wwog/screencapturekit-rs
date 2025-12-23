use std::env;
use std::process::Command;

fn main() {
    println!("cargo:rustc-link-lib=framework=ScreenCaptureKit");

    // Build the Swift bridge
    let swift_dir = "swift-bridge";
    let out_dir = env::var("OUT_DIR").unwrap();
    let swift_build_dir = format!("{out_dir}/swift-build");

    println!("cargo:rerun-if-changed={swift_dir}");

    // Run swiftlint if available (non-strict mode, don't fail build)
    if let Ok(output) = Command::new("swiftlint")
        .args(["lint"])
        .current_dir(swift_dir)
        .output()
    {
        if !output.status.success() {
            eprintln!(
                "SwiftLint warnings:\n{}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
    }

    let feature_open_15 = env::var("CARGO_FEATURE_MACOS_15_0").is_ok();
    let feature_open_26 = env::var("CARGO_FEATURE_MACOS_26_0").is_ok();
    
    // Detect target architecture for cross-compilation support
    let target = env::var("TARGET").unwrap_or_default();
    let host = env::var("HOST").unwrap_or_default();
    let use_rosetta = target.contains("x86_64") && (host.contains("aarch64") || host.contains("arm64"));
    
    let swift_target_triple = if target.contains("x86_64") {
        Some("x86_64-apple-macosx")
    } else if target.contains("aarch64") || target.contains("arm64") {
        Some("arm64-apple-macosx")
    } else {
        None
    };
    
    if let Some(triple) = swift_target_triple {
        println!("cargo:warning=Building Swift bridge for target: {triple} (Rust target: {target})");
    }
    
    let mut args = vec![
        "build",
        "-c",
        "release",
        "--package-path",
        swift_dir,
        "--scratch-path",
        &swift_build_dir,
    ];

    // Add target triple for cross-compilation
    // Note: When using arch -x86_64, Swift automatically handles the architecture
    // so we don't need to add -target flags. For native builds, we also skip
    // -target flags as Swift Package Manager handles this automatically.
    // The architecture is determined by the build environment.

    if feature_open_15 {
        args.push("--features");
        args.push("macos_15_0");
    }
    if feature_open_26 {
        args.push("--features");
        args.push("macos_26_0");
    }

    // Build Swift package with build directory in OUT_DIR
    // For cross-compilation from ARM to x86_64, use arch -x86_64
    
    println!("cargo:warning=Building Swift bridge (target: {target}, host: {host})");
    
    let output = if use_rosetta {
        println!("cargo:warning=Cross-compiling to x86_64 on ARM Mac, using arch -x86_64");
        // Use arch -x86_64 to run swift in x86_64 mode via Rosetta 2
        // Format: arch -x86_64 swift build ...
        Command::new("arch")
            .arg("-x86_64")
            .arg("swift")
            .args(&args)
            .output()
            .expect("Failed to build Swift bridge with arch -x86_64")
    } else {
        Command::new("swift")
            .args(&args)
            .output()
            .expect("Failed to build Swift bridge")
    };

    // Swift build outputs warnings to stderr even on success, check exit code only
    if !output.status.success() {
        eprintln!(
            "Swift build STDOUT:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "Swift build STDERR:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        panic!(
            "Swift build failed with exit code: {:?}",
            output.status.code()
        );
    }

    // Link the Swift library from OUT_DIR
    println!("cargo:rustc-link-search=native={swift_build_dir}/release");
    println!("cargo:rustc-link-lib=static=ScreenCaptureKitBridge");

    // Link required frameworks
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=CoreGraphics");
    println!("cargo:rustc-link-lib=framework=CoreMedia");
    println!("cargo:rustc-link-lib=framework=IOSurface");

    // Add rpath for Swift runtime libraries
    // For x86_64, use x86_64 subdirectory; for arm64, use arm64 or default
    let swift_runtime_arch = if target.contains("x86_64") {
        "x86_64"
    } else {
        "arm64"
    };
    
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    
    // Add architecture-specific Swift runtime path
    let swift_arch_path = format!("/usr/lib/swift/{swift_runtime_arch}");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{swift_arch_path}");

    // Add rpath for Xcode Swift runtime (needed for Swift Concurrency)
    if let Ok(output) = Command::new("xcode-select").arg("-p").output() {
        if output.status.success() {
            let xcode_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            
            // Add architecture-specific paths
            let swift_lib_path_arch = format!(
                "{xcode_path}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx/{swift_runtime_arch}"
            );
            println!("cargo:rustc-link-arg=-Wl,-rpath,{swift_lib_path_arch}");
            
            let swift_lib_path = format!(
                "{xcode_path}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx"
            );
            println!("cargo:rustc-link-arg=-Wl,-rpath,{swift_lib_path}");
            
            // Also add the newer swift path
            let swift_lib_path_new_arch = format!(
                "{xcode_path}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx/{swift_runtime_arch}"
            );
            println!("cargo:rustc-link-arg=-Wl,-rpath,{swift_lib_path_new_arch}");
            
            let swift_lib_path_new =
                format!("{xcode_path}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{swift_lib_path_new}");
        }
    }
}
