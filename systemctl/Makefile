export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup

OUTPUT_DIR = $(DADK_BUILD_CACHE_DIR_DRAGONREACH_0_1_0)
REACH_BIN_DIR=$(OUTPUT_DIR)/bin/
TMP_INSTALL_DIR=$(OUTPUT_DIR)/tmp_install

all: build

build:
	cargo +nightly-2023-08-15 -Z build-std=core,alloc,compiler_builtins build --target ../x86_64-unknown-dragonos.json --release

install:
	mkdir -p $(TMP_INSTALL_DIR)
	mkdir -p $(REACH_BIN_DIR)

	cargo -Z build-std=core,alloc,compiler_builtins install --target $(TARGET) --path .  --root $(TMP_INSTALL_DIR)
	mv $(TMP_INSTALL_DIR)/bin/systemctl $(REACH_BIN_DIR)/systemctl
	rm -rf $(TMP_INSTALL_DIR)

build-linux:
	cargo +nightly-2023-08-15 -Z build-std=core,alloc,compiler_builtins build --target x86_64-unknown-linux-gnu

clean:
	cargo clean

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

check:
	cargo +nightly-2023-08-15 -Z build-std=core,alloc,compiler_builtins check --workspace --message-format=json --target ../x86_64-unknown-dragonos.json
