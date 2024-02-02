# The toolchain we use.
# You can get it by running DragonOS' `tools/bootstrap.sh`
TOOLCHAIN="+nightly-2023-08-15-x86_64-unknown-linux-gnu"
RUSTFLAGS+="-C target-feature=+crt-static -C link-arg=-no-pie"

# 如果是在dadk中编译，那么安装到dadk的安装目录中
INSTALL_DIR?=$(DADK_CURRENT_BUILD_DIR)
# 如果是在本地编译，那么安装到当前目录下的install目录中
INSTALL_DIR?=./install


build:
	RUSTFLAGS=$(RUSTFLAGS) cargo $(TOOLCHAIN) build

run-dragonreach:
	RUSTFLAGS=$(RUSTFLAGS) cargo $(TOOLCHAIN) run --bin DragonReach

clean:
	RUSTFLAGS=$(RUSTFLAGS) cargo $(TOOLCHAIN) clean

build-release:
	RUSTFLAGS=$(RUSTFLAGS) cargo $(TOOLCHAIN) build --release

clean-release:
	RUSTFLAGS=$(RUSTFLAGS) cargo $(TOOLCHAIN) clean --release

fmt:
	RUSTFLAGS=$(RUSTFLAGS) cargo $(TOOLCHAIN) fmt

fmt-check:
	RUSTFLAGS=$(RUSTFLAGS) cargo $(TOOLCHAIN) fmt --check

.PHONY: install
install:
	mkdir -p $(INSTALL_DIR)/etc/reach/system
	mkdir -p $(INSTALL_DIR)/etc/reach/ipc
	cp ./parse_test/shell.service $(INSTALL_DIR)/etc/reach/system/shell.service
	RUSTFLAGS=$(RUSTFLAGS) cargo $(TOOLCHAIN) install --path . --no-track --root $(INSTALL_DIR) --force
