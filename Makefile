IMAGE_NAME ?= chinese-poetry-backend
IMAGE_TAG ?= latest
PLATFORM ?= linux/amd64

.PHONY: build run docker-build docker-push clean dev-frontend dev-backend dev

# ---- 本地开发 ----

dev-frontend:
	cd frontend && pnpm dev

dev-backend:
	cargo run

# 同时启动前后端（需要两个终端）
dev:
	@echo "请分别在两个终端运行:"
	@echo "  make dev-frontend"
	@echo "  make dev-backend"

# ---- 前端构建（本地） ----

frontend-build:
	cd frontend && CI=true pnpm install && pnpm build

# ---- Docker 构建 ----

# 全部在 Docker 内完成（前端构建 + Rust 编译）
docker-build:
	docker build --platform $(PLATFORM) -t $(IMAGE_NAME):$(IMAGE_TAG) .
	@echo "构建完成: $(IMAGE_NAME):$(IMAGE_TAG) ($(PLATFORM))"

docker-build-no-cache:
	docker build --platform $(PLATFORM) --no-cache -t $(IMAGE_NAME):$(IMAGE_TAG) .
	@echo "构建完成 (no-cache): $(IMAGE_NAME):$(IMAGE_TAG) ($(PLATFORM))"

# ---- Docker 运行 ----

docker-run:
	docker run -d --name poetry-backend \
		-p 3000:3000 \
		-v $$(pwd)/config.toml:/app/config.toml:ro \
		$(IMAGE_NAME):$(IMAGE_TAG)

docker-stop:
	docker stop poetry-backend && docker rm poetry-backend

# ---- Docker 推送 ----

docker-push: docker-build
	docker push $(IMAGE_NAME):$(IMAGE_TAG)

# ---- 清理 ----

clean:
	cd frontend && rm -rf dist node_modules
	cargo clean
