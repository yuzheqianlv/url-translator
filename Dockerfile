FROM rust:1.88 as builder

WORKDIR /app

# 安装构建依赖
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk

# 先复制依赖文件，利用Docker缓存
COPY Cargo.toml Cargo.lock ./
COPY Trunk.toml ./

# 创建src目录结构（用于依赖缓存）
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# 预构建依赖
RUN cargo build --release --target wasm32-unknown-unknown

# 复制所有源代码和资源文件
COPY src/ ./src/
COPY index.html ./
COPY style.css ./

# 构建应用
RUN trunk build --release

# 生产镜像
FROM nginx:alpine

# 复制构建文件
COPY --from=builder /app/dist /usr/share/nginx/html

# 创建nginx配置
RUN echo 'server { \
    listen 80; \
    server_name localhost; \
    root /usr/share/nginx/html; \
    index index.html; \
    \
    # 处理SPA路由 \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
    \
    # 静态资源缓存 \
    location ~* \.(js|css|wasm|png|jpg|jpeg|gif|ico|svg)$ { \
        expires 1y; \
        add_header Cache-Control "public, immutable"; \
    } \
    \
    # 安全头 \
    add_header X-Frame-Options "SAMEORIGIN" always; \
    add_header X-Content-Type-Options "nosniff" always; \
    add_header Referrer-Policy "no-referrer-when-downgrade" always; \
    \
    # CORS支持 \
    add_header Access-Control-Allow-Origin "*" always; \
    add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always; \
    add_header Access-Control-Allow-Headers "Origin, Content-Type, Accept, Authorization" always; \
}' > /etc/nginx/conf.d/default.conf

# 暴露端口
EXPOSE 80

# 启动nginx
CMD ["nginx", "-g", "daemon off;"]