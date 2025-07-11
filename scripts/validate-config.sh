#!/bin/bash
# 配置验证脚本

set -e

echo "🔍 验证前端配置..."

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 验证函数
validate_url() {
    local url=$1
    local name=$2
    
    if [[ $url =~ ^https?:// ]]; then
        echo -e "${GREEN}✅ $name: $url${NC}"
        return 0
    else
        echo -e "${RED}❌ $name: 无效的URL格式: $url${NC}"
        return 1
    fi
}

validate_number() {
    local value=$1
    local name=$2
    local min=$3
    local max=$4
    
    if [[ $value =~ ^[0-9]+$ ]] && [ $value -ge $min ] && [ $value -le $max ]; then
        echo -e "${GREEN}✅ $name: $value${NC}"
        return 0
    else
        echo -e "${RED}❌ $name: 值应该在 $min-$max 之间，当前值: $value${NC}"
        return 1
    fi
}

validate_boolean() {
    local value=$1
    local name=$2
    
    if [[ $value == "true" || $value == "false" ]]; then
        echo -e "${GREEN}✅ $name: $value${NC}"
        return 0
    else
        echo -e "${RED}❌ $name: 应该是 true 或 false，当前值: $value${NC}"
        return 1
    fi
}

validate_theme() {
    local theme=$1
    local valid_themes=("latte" "frappe" "macchiato" "mocha")
    
    for valid_theme in "${valid_themes[@]}"; do
        if [[ $theme == $valid_theme ]]; then
            echo -e "${GREEN}✅ DEFAULT_THEME: $theme${NC}"
            return 0
        fi
    done
    
    echo -e "${RED}❌ DEFAULT_THEME: 无效的主题 '$theme'，支持的主题: ${valid_themes[*]}${NC}"
    return 1
}

# 加载环境变量
if [ -f ".env" ]; then
    echo "📄 加载 .env 配置..."
    source .env
else
    echo -e "${YELLOW}⚠️ .env 文件不存在，使用默认值${NC}"
fi

if [ -f ".env.local" ]; then
    echo "📄 加载 .env.local 配置..."
    source .env.local
fi

# 设置默认值
FRONTEND_API_BASE_URL=${FRONTEND_API_BASE_URL:-"http://localhost:3002/api/v1"}
FRONTEND_API_TIMEOUT_SECONDS=${FRONTEND_API_TIMEOUT_SECONDS:-30}
ENABLE_PROJECT_MANAGEMENT=${ENABLE_PROJECT_MANAGEMENT:-true}
ENABLE_HISTORY=${ENABLE_HISTORY:-true}
ENABLE_SEARCH=${ENABLE_SEARCH:-true}
ENABLE_BATCH_TRANSLATION=${ENABLE_BATCH_TRANSLATION:-true}
DEFAULT_THEME=${DEFAULT_THEME:-latte}
DEBUG_MODE=${DEBUG_MODE:-true}
MAX_FILE_SIZE_MB=${MAX_FILE_SIZE_MB:-10}

echo ""
echo "🔧 验证配置项..."

# 验证计数器
valid_count=0
total_count=0

# API配置验证
echo ""
echo "📡 API 配置:"
validate_url "$FRONTEND_API_BASE_URL" "FRONTEND_API_BASE_URL" && ((valid_count++))
((total_count++))

validate_number "$FRONTEND_API_TIMEOUT_SECONDS" "FRONTEND_API_TIMEOUT_SECONDS" 1 300 && ((valid_count++))
((total_count++))

# 功能开关验证
echo ""
echo "🎛️ 功能开关:"
validate_boolean "$ENABLE_PROJECT_MANAGEMENT" "ENABLE_PROJECT_MANAGEMENT" && ((valid_count++))
((total_count++))

validate_boolean "$ENABLE_HISTORY" "ENABLE_HISTORY" && ((valid_count++))
((total_count++))

validate_boolean "$ENABLE_SEARCH" "ENABLE_SEARCH" && ((valid_count++))
((total_count++))

validate_boolean "$ENABLE_BATCH_TRANSLATION" "ENABLE_BATCH_TRANSLATION" && ((valid_count++))
((total_count++))

validate_boolean "$DEBUG_MODE" "DEBUG_MODE" && ((valid_count++))
((total_count++))

# UI 配置验证
echo ""
echo "🎨 UI 配置:"
validate_theme "$DEFAULT_THEME" && ((valid_count++))
((total_count++))

validate_number "$MAX_FILE_SIZE_MB" "MAX_FILE_SIZE_MB" 1 100 && ((valid_count++))
((total_count++))

# 网络连接测试
echo ""
echo "🌐 网络连接测试:"
if curl -s --max-time 5 "$FRONTEND_API_BASE_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ 后端API连接正常: $FRONTEND_API_BASE_URL${NC}"
    ((valid_count++))
else
    echo -e "${YELLOW}⚠️ 后端API连接失败: $FRONTEND_API_BASE_URL${NC}"
    echo -e "${YELLOW}   这可能是正常的（如果后端未启动）${NC}"
fi
((total_count++))

# 构建工具检查
echo ""
echo "🛠️ 构建工具检查:"
if command -v trunk &> /dev/null; then
    echo -e "${GREEN}✅ Trunk: $(trunk --version)${NC}"
    ((valid_count++))
else
    echo -e "${RED}❌ Trunk 未安装${NC}"
fi
((total_count++))

if rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo -e "${GREEN}✅ WASM 目标已安装${NC}"
    ((valid_count++))
else
    echo -e "${RED}❌ WASM 目标未安装${NC}"
fi
((total_count++))

# 文件检查
echo ""
echo "📁 文件检查:"
required_files=("Cargo.toml" "index.html" "src/main.rs" "src/lib.rs")
for file in "${required_files[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✅ $file 存在${NC}"
        ((valid_count++))
    else
        echo -e "${RED}❌ $file 缺失${NC}"
    fi
    ((total_count++))
done

# 总结
echo ""
echo "📊 验证结果:"
echo "========================================"
echo "有效配置: $valid_count / $total_count"

if [ $valid_count -eq $total_count ]; then
    echo -e "${GREEN}🎉 所有配置都有效！${NC}"
    exit 0
elif [ $valid_count -gt $((total_count * 3 / 4)) ]; then
    echo -e "${YELLOW}⚠️ 大部分配置有效，但有一些问题需要注意${NC}"
    exit 1
else
    echo -e "${RED}❌ 存在多个配置问题，请修复后重试${NC}"
    exit 2
fi