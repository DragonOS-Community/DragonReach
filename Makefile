# The toolchain we use.
# You can get it by running DragonOS' `tools/bootstrap.sh`
TOOLCHAIN="+nightly-2023-08-15-x86_64-unknown-linux-gnu"


ifdef DADK_CURRENT_BUILD_DIR
# 如果是在dadk中编译，那么安装到dadk的安装目录中
	INSTALL_DIR = $(DADK_CURRENT_BUILD_DIR)
else
# 如果是在本地编译，那么安装到当前目录下的install目录中
	INSTALL_DIR = ./install
endif


ifeq ($(ARCH), x86_64)
	export RUST_TARGET=x86_64-unknown-linux-musl
else ifeq ($(ARCH), riscv64)
	export RUST_TARGET=riscv64gc-unknown-linux-gnu
else 
# 默认为x86_86，用于本地编译
	export RUST_TARGET=x86_64-unknown-linux-musl
endif

build:
	cargo $(TOOLCHAIN) build --target $(RUST_TARGET)

run-dragonreach:
	cargo $(TOOLCHAIN) run --target $(RUST_TARGET) --bin DragonReach

clean:
	cargo $(TOOLCHAIN) clean

build-release:
	cargo $(TOOLCHAIN) build --target $(RUST_TARGET) --release

clean-release:
	cargo $(TOOLCHAIN) clean --target $(RUST_TARGET) --release

fmt:
	cargo $(TOOLCHAIN) fmt

fmt-check:
	cargo $(TOOLCHAIN) fmt --check

.PHONY: install
install:
	mkdir -p $(INSTALL_DIR)/etc/reach/system
	mkdir -p $(INSTALL_DIR)/etc/reach/ipc
	cp ./parse_test/shell.service $(INSTALL_DIR)/etc/reach/system/shell.service
	cargo $(TOOLCHAIN) install --target $(RUST_TARGET) --path . --no-track --root $(INSTALL_DIR) --force