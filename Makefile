OUTPUT_DIR = $(DADK_BUILD_CACHE_DIR_DRAGONREACH_0_1_0)
REACH_ETC_DIR=$(OUTPUT_DIR)/etc/reach
REACH_BIN_DIR=$(OUTPUT_DIR)/bin/
TMP_INSTALL_DIR=$(OUTPUT_DIR)/tmp_install

build:
	cargo -Z build-std=core,alloc,compiler_builtins build --target ./target.json --release

install:
	mkdir -p $(TMP_INSTALL_DIR)
	mkdir -p $(REACH_ETC_DIR)
	mkdir -p $(REACH_ETC_DIR)/system/
	mkdir -p $(REACH_BIN_DIR)

	cp ./parse_test/shell.service $(REACH_ETC_DIR)/system/shell.service

	cargo -Z build-std=core,alloc,compiler_builtins install --target $(TARGET) --path .  --root $(TMP_INSTALL_DIR)
	mv $(OUTPUT_DIR)/tmp/bin/DragonReach $(REACH_BIN_DIR)/DragonReach
	rm -rf $(TMP_INSTALL_DIR)

build-linux:
	cargo -Z build-std=core,alloc,compiler_builtins build --target x86_64-unknown-linux-gnu

clean:
	cargo clean
