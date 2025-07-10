-- URL翻译工具数据库初始化脚本
-- Database Initialization Script for URL Translator

-- 设置时区和编码
SET timezone = 'UTC';
SET client_encoding = 'UTF8';

-- =============================================================================
-- 用户和配置表 - Users and Configuration Tables
-- =============================================================================

-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    last_login_at TIMESTAMP WITH TIME ZONE
);

-- 用户配置表
CREATE TABLE IF NOT EXISTS user_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    deeplx_api_url TEXT,
    jina_api_url TEXT DEFAULT 'https://r.jina.ai',
    default_source_lang VARCHAR(10) DEFAULT 'auto',
    default_target_lang VARCHAR(10) DEFAULT 'zh',
    max_requests_per_second INTEGER DEFAULT 10,
    max_text_length INTEGER DEFAULT 5000,
    max_paragraphs_per_request INTEGER DEFAULT 10,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id)
);

-- =============================================================================
-- 翻译相关表 - Translation Related Tables
-- =============================================================================

-- 翻译项目表 (用于批量翻译)
CREATE TABLE IF NOT EXISTS translation_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'cancelled')),
    total_urls INTEGER DEFAULT 0,
    completed_urls INTEGER DEFAULT 0,
    failed_urls INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE
);

-- 翻译记录表
CREATE TABLE IF NOT EXISTS translation_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES translation_projects(id) ON DELETE CASCADE,
    original_url TEXT NOT NULL,
    title TEXT,
    source_lang VARCHAR(10),
    target_lang VARCHAR(10),
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    error_message TEXT,
    word_count INTEGER DEFAULT 0,
    character_count INTEGER DEFAULT 0,
    processing_time_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE
);

-- 翻译内容表 (存储实际的翻译内容)
CREATE TABLE IF NOT EXISTS translation_contents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    record_id UUID REFERENCES translation_records(id) ON DELETE CASCADE UNIQUE,
    original_content TEXT NOT NULL,
    translated_content TEXT,
    content_hash VARCHAR(64), -- SHA-256 hash for deduplication
    file_name VARCHAR(255),
    file_size INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- =============================================================================
-- 搜索和索引相关表 - Search and Index Related Tables
-- =============================================================================

-- 搜索历史表
CREATE TABLE IF NOT EXISTS search_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    query TEXT NOT NULL,
    filters JSONB,
    results_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- =============================================================================
-- 系统表 - System Tables
-- =============================================================================

-- API使用统计表
CREATE TABLE IF NOT EXISTS api_usage_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    service_name VARCHAR(50) NOT NULL, -- 'jina', 'deeplx', etc.
    endpoint TEXT,
    status_code INTEGER,
    response_time_ms INTEGER,
    request_size INTEGER,
    response_size INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 系统配置表
CREATE TABLE IF NOT EXISTS system_configs (
    key VARCHAR(100) PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- =============================================================================
-- 索引创建 - Index Creation
-- =============================================================================

-- 用户相关索引
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at);

-- 翻译记录索引
CREATE INDEX IF NOT EXISTS idx_translation_records_user_id ON translation_records(user_id);
CREATE INDEX IF NOT EXISTS idx_translation_records_project_id ON translation_records(project_id);
CREATE INDEX IF NOT EXISTS idx_translation_records_status ON translation_records(status);
CREATE INDEX IF NOT EXISTS idx_translation_records_created_at ON translation_records(created_at);
CREATE INDEX IF NOT EXISTS idx_translation_records_url_hash ON translation_records(md5(original_url));

-- 翻译项目索引
CREATE INDEX IF NOT EXISTS idx_translation_projects_user_id ON translation_projects(user_id);
CREATE INDEX IF NOT EXISTS idx_translation_projects_status ON translation_projects(status);
CREATE INDEX IF NOT EXISTS idx_translation_projects_created_at ON translation_projects(created_at);

-- 翻译内容索引
CREATE INDEX IF NOT EXISTS idx_translation_contents_record_id ON translation_contents(record_id);
CREATE INDEX IF NOT EXISTS idx_translation_contents_hash ON translation_contents(content_hash);

-- 搜索历史索引
CREATE INDEX IF NOT EXISTS idx_search_history_user_id ON search_history(user_id);
CREATE INDEX IF NOT EXISTS idx_search_history_created_at ON search_history(created_at);

-- API统计索引
CREATE INDEX IF NOT EXISTS idx_api_usage_stats_user_id ON api_usage_stats(user_id);
CREATE INDEX IF NOT EXISTS idx_api_usage_stats_service ON api_usage_stats(service_name);
CREATE INDEX IF NOT EXISTS idx_api_usage_stats_created_at ON api_usage_stats(created_at);

-- =============================================================================
-- 触发器和函数 - Triggers and Functions
-- =============================================================================

-- 更新时间戳触发器函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 为需要的表添加更新时间戳触发器
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_configs_updated_at BEFORE UPDATE ON user_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_translation_projects_updated_at BEFORE UPDATE ON translation_projects
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_translation_records_updated_at BEFORE UPDATE ON translation_records
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_system_configs_updated_at BEFORE UPDATE ON system_configs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =============================================================================
-- 初始数据插入 - Initial Data Insertion
-- =============================================================================

-- 插入默认系统配置
INSERT INTO system_configs (key, value, description) VALUES
    ('app_version', '2.0.0', 'Application version'),
    ('maintenance_mode', 'false', 'Maintenance mode flag'),
    ('max_file_size_mb', '50', 'Maximum file size in MB'),
    ('default_cache_ttl_hours', '24', 'Default cache TTL in hours'),
    ('meilisearch_index_prefix', 'url_translator_', 'MeiliSearch index prefix')
ON CONFLICT (key) DO NOTHING;

-- 创建默认的匿名用户配置 (用于未登录用户)
INSERT INTO users (id, username, email, password_hash, is_active) VALUES
    ('00000000-0000-0000-0000-000000000000', 'anonymous', 'anonymous@localhost', '', false)
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_configs (user_id, deeplx_api_url, jina_api_url) VALUES
    ('00000000-0000-0000-0000-000000000000', 'http://localhost:1188/translate', 'https://r.jina.ai')
ON CONFLICT (user_id) DO NOTHING;

-- =============================================================================
-- 权限设置 - Permissions Setup
-- =============================================================================

-- 创建只读用户 (用于备份和分析)
-- CREATE USER readonly_user WITH PASSWORD 'readonly_password';
-- GRANT CONNECT ON DATABASE markdown_manager TO readonly_user;
-- GRANT USAGE ON SCHEMA public TO readonly_user;
-- GRANT SELECT ON ALL TABLES IN SCHEMA public TO readonly_user;
-- ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO readonly_user;

-- 完成初始化
INSERT INTO system_configs (key, value, description) VALUES
    ('db_initialized_at', EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)::TEXT, 'Database initialization timestamp')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = CURRENT_TIMESTAMP;

-- 显示初始化完成信息
DO $$
BEGIN
    RAISE NOTICE 'URL Translator Database initialized successfully!';
    RAISE NOTICE 'Tables created: %', (
        SELECT COUNT(*) FROM information_schema.tables 
        WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
    );
    RAISE NOTICE 'Indexes created: %', (
        SELECT COUNT(*) FROM pg_indexes WHERE schemaname = 'public'
    );
END $$;