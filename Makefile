# OpenIoT 项目 Makefile
# 适用于 Windows (需要 make 工具，如 MSYS2/Git Bash)

.PHONY: all build build-server build-client run-server run-client clean deploy-esp32 help

# 默认目标
all: build

# 编译整个 workspace
build:
	cargo build --workspace

# 编译服务端
build-server:
	cargo build -p openiot-server

# 编译 Windows 客户端
build-client:
	cargo build -p openiot-client

# 运行服务端
run-server:
	cargo run -p openiot-server

# 运行 Windows 客户端
run-client:
	cargo run -p openiot-client

# 清理构建产物
clean:
	cargo clean

# 部署 ESP32 固件（需要安装 mpremote）
deploy-esp32:
	mpremote connect auto cp esp32/main.py esp32/config.py esp32/provision.py esp32/protocol.py esp32/mesh.py esp32/device.py :

# 仅部署主程序
deploy-esp32-main:
	mpremote connect auto cp esp32/main.py :

# 重启 ESP32
restart-esp32:
	mpremote connect auto reset

# 查看 ESP32 REPL
repl-esp32:
	mpremote connect auto repl

# 帮助
help:
	@echo "OpenIoT 构建命令:"
	@echo "  make build          - 编译整个项目"
	@echo "  make build-server   - 编译服务端"
	@echo "  make build-client   - 编译 Windows 客户端"
	@echo "  make run-server     - 运行服务端 (http://localhost:3000)"
	@echo "  make run-client     - 运行 Windows 客户端"
	@echo "  make clean          - 清理构建产物"
	@echo "  make deploy-esp32   - 部署 ESP32 固件"
	@echo "  make restart-esp32  - 重启 ESP32"
	@echo "  make repl-esp32     - 进入 ESP32 REPL"
