# ---- Stage 1: Build frontend ----
FROM node:20-alpine AS frontend-builder

WORKDIR /app/frontend
COPY frontend/package.json frontend/pnpm-lock.yaml* ./
RUN npm install -g pnpm && pnpm install --frozen-lockfile || pnpm install
COPY frontend/ ./
RUN pnpm build

# ---- Stage 2: Build Rust binary (with embedded frontend) ----
FROM rust:1.88-bookworm AS backend-builder

WORKDIR /app
# 先复制依赖文件，利用 Docker 层缓存
COPY Cargo.toml Cargo.lock ./
# 创建空 src 目录以缓存依赖编译
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# 复制完整源码
COPY src/ src/
COPY migrations/ migrations/
# 复制前端编译产物到 rust-embed 期望的路径
COPY --from=frontend-builder /app/frontend/dist/ frontend/dist/

# 重新编译（只重编译项目代码，依赖已缓存）
RUN touch src/main.rs && cargo build --release

# ---- Stage 3: Runtime ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend-builder /app/target/release/chinese-poetry-backend /app/chinese-poetry-backend

EXPOSE 3000

ENTRYPOINT ["/app/chinese-poetry-backend"]
