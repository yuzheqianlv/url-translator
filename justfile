# Justfile for URL Translator project
# 使用 just 命令运行各种开发任务

# 默认任务
default:
    @just --list

# 开发服务器
dev:
    ./scripts/dev.sh

# 开发服务器（简单模式）
dev-simple:
    trunk serve --open

# 构建项目
build:
    trunk build --release

# 生产构建
build-prod:
    ./scripts/build-prod.sh

# 运行所有测试
test:
    cargo test --lib
    wasm-pack test --headless --chrome

# 运行单元测试
test-unit:
    cargo test --lib

# 运行WASM测试
test-wasm:
    wasm-pack test --headless --chrome

# 运行特定测试
test-specific TEST_NAME:
    cargo test {{TEST_NAME}}

# 代码格式化
fmt:
    cargo fmt

# 检查代码格式
fmt-check:
    cargo fmt -- --check

# 运行clippy
clippy:
    cargo clippy -- -D warnings

# 检查代码（快速）
check:
    cargo check

# 完整的代码质量检查
quality:
    cargo fmt -- --check
    cargo clippy -- -D warnings
    cargo test --lib
    wasm-pack test --headless --chrome

# 清理构建缓存
clean:
    cargo clean
    rm -rf dist/

# 更新依赖
update:
    cargo update

# 生成文档
doc:
    cargo doc --open

# 代码覆盖率
coverage:
    cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
    @echo "Coverage report generated: lcov.info"

# 安装开发依赖
install-deps:
    cargo install trunk
    cargo install wasm-pack
    cargo install cargo-llvm-cov
    rustup target add wasm32-unknown-unknown

# 项目初始化
init:
    @just install-deps
    @echo "复制环境变量配置文件..."
    @cp .env.example .env || echo ".env 文件已存在"
    @echo "开发环境已初始化"
    @echo "请编辑 .env 文件配置您的开发环境"

# 环境配置
setup-env:
    @echo "设置环境配置..."
    @cp .env.example .env || echo ".env 文件已存在"
    @cp .env.local.example .env.local || echo ".env.local 文件已存在"
    @echo "环境配置文件已创建，请根据需要修改配置"

# 验证配置
validate:
    ./scripts/validate-config.sh

# 检查环境
check-env:
    @echo "检查开发环境..."
    @which trunk || echo "❌ Trunk 未安装"
    @which rustc || echo "❌ Rust 未安装"
    @rustup target list --installed | grep wasm32-unknown-unknown || echo "❌ WASM 目标未安装"
    @echo "✅ 环境检查完成"

# 发布构建
release:
    trunk build --release
    @echo "发布构建完成，文件位于 dist/ 目录"

# 运行性能测试
bench:
    cargo bench

# 监视文件变化并运行测试
watch-test:
    cargo watch -x "test --lib"

# 监视文件变化并运行检查
watch-check:
    cargo watch -x check

# 生成测试报告
test-report:
    cargo test --lib -- --nocapture
    wasm-pack test --headless --chrome -- --nocapture