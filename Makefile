export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup

OUTPUT_DIR = $(DADK_BUILD_CACHE_DIR_DRAGONREACH_0_1_0)
REACH_ETC_DIR=$(OUTPUT_DIR)/etc/reach
REACH_BIN_DIR=$(OUTPUT_DIR)/bin/
TMP_INSTALL_DIR=$(OUTPUT_DIR)/tmp_install

all: build

build:
	@$(MAKE) -C ./systemctl build
	cargo +nightly-2023-08-15 -Z build-std=core,alloc,compiler_builtins build --target ./x86_64-unknown-dragonos.json --release

install:
	mkdir -p $(TMP_INSTALL_DIR)
	mkdir -p $(REACH_ETC_DIR)
	mkdir -p $(REACH_ETC_DIR)/system/
	mkdir -p $(REACH_BIN_DIR)
	mkdir -p $(REACH_ETC_DIR)/ipc/

	cp ./parse_test/shell.service $(REACH_ETC_DIR)/system/shell.service

	cargo +nightly-2023-08-15 -Z build-std=core,alloc,compiler_builtins install --target $(TARGET) --path .  --root $(TMP_INSTALL_DIR)
	mv $(TMP_INSTALL_DIR)/bin/DragonReach $(REACH_BIN_DIR)/DragonReach
	
	cargo -Z build-std=core,alloc,compiler_builtins install --target $(TARGET) --path ./systemctl  --root $(TMP_INSTALL_DIR)
	mv $(TMP_INSTALL_DIR)/bin/systemctl $(REACH_BIN_DIR)/systemctl
	
	rm -rf $(TMP_INSTALL_DIR)

build-linux:
	@$(MAKE) -C ./systemctl build-linux
	cargo -Z build-std=core,alloc,compiler_builtins build --target x86_64-unknown-linux-gnu

clean:
	cargo clean
	@$(MAKE) -C ./systemctl clean

fmt:
	cargo fmt
	@$(MAKE) -C ./systemctl fmt

fmt-check:
	cargo fmt --check
	@$(MAKE) -C ./systemctl fmt-check

check:
	cargo -Z build-std=core,alloc,compiler_builtins check --workspace --message-format=json --target ./x86_64-unknown-dragonos.json
	@$(MAKE) -C ./systemctl check
