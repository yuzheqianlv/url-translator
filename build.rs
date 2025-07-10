//! 构建脚本
//! 
//! 处理环境变量和构建时配置

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=.env.local");
    println!("cargo:rerun-if-env-changed=FRONTEND_API_BASE_URL");
    println!("cargo:rerun-if-env-changed=FRONTEND_API_TIMEOUT_SECONDS");

    // 生成构建时间戳
    let build_timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);

    // 加载.env文件（如果存在）
    if Path::new(".env").exists() {
        load_env_file(".env");
    }

    // 加载.env.local文件（如果存在，优先级更高）
    if Path::new(".env.local").exists() {
        load_env_file(".env.local");
    }

    // 验证必要的环境变量
    validate_env_vars();

    // 生成功能开关配置
    generate_feature_config();

    println!("cargo:rustc-env=CARGO_PKG_BUILD_TARGET=wasm32-unknown-unknown");
}

/// 加载.env文件中的环境变量
fn load_env_file(file_path: &str) {
    if let Ok(content) = fs::read_to_string(file_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');
                
                // 只设置未设置的环境变量
                if env::var(key).is_err() {
                    println!("cargo:rustc-env={}={}", key, value);
                }
            }
        }
    }
}

/// 验证必要的环境变量
fn validate_env_vars() {
    let required_vars = [
        ("FRONTEND_API_BASE_URL", "http://localhost:3002/api/v1"),
        ("FRONTEND_API_TIMEOUT_SECONDS", "30"),
        ("DEFAULT_THEME", "latte"),
    ];

    for (var_name, default_value) in required_vars.iter() {
        if env::var(var_name).is_err() {
            println!("cargo:rustc-env={}={}", var_name, default_value);
            println!("cargo:warning=环境变量 {} 未设置，使用默认值: {}", var_name, default_value);
        }
    }
}

/// 生成功能开关配置
fn generate_feature_config() {
    let features = [
        ("ENABLE_PROJECT_MANAGEMENT", "true"),
        ("ENABLE_HISTORY", "true"),
        ("ENABLE_SEARCH", "true"),
        ("ENABLE_BATCH_TRANSLATION", "true"),
        ("ENABLE_USER_AUTHENTICATION", "true"),
        ("DEBUG_MODE", "true"),
    ];

    for (feature_name, default_value) in features.iter() {
        if env::var(feature_name).is_err() {
            println!("cargo:rustc-env={}={}", feature_name, default_value);
        }
    }

    // 生成构建信息
    let build_profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    println!("cargo:rustc-env=BUILD_PROFILE={}", build_profile);

    // 检查是否为生产构建
    let is_release = build_profile == "release";
    println!("cargo:rustc-env=IS_RELEASE_BUILD={}", is_release);

    // 生成版本信息
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string());
    let git_hash = get_git_hash().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_VERSION={}", version);
    println!("cargo:rustc-env=BUILD_GIT_HASH={}", git_hash);
}

/// 获取Git提交哈希
fn get_git_hash() -> Option<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        let hash = String::from_utf8(output.stdout).ok()?;
        Some(hash.trim().to_string())
    } else {
        None
    }
}