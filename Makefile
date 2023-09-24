OUTPUT_DIR = $(DADK_BUILD_CACHE_DIR_DRAGONREACH_0_1_0)

build:
	cargo -Z build-std=core,alloc,compiler_builtins build --target ./target.json

install:
	cp ./parse_test/shell.service $(ROOT_PATH)/bin/sysroot/etc/reach/system/shell.service

	mkdir -p $(OUTPUT_DIR)/tmp
	cargo -Z build-std=core,alloc,compiler_builtins install --target $(TARGET) --path .  --root $(OUTPUT_DIR)/tmp
	mv $(OUTPUT_DIR)/tmp/bin/DragonReach $(ROOT_PATH)/bin/user/DragonReach
	rm -rf $(OUTPUT_DIR)/tmp

build-linux:
	cargo -Z build-std=core,alloc,compiler_builtins build --target x86_64-unknown-linux-gnu

clean:
	cargo clean
