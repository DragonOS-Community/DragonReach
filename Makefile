build:
	cargo -Z build-std=core,alloc,compiler_builtins build --target x86_64-unknown-dragonos

install:
	cargo -Z build-std=core,alloc,compiler_builtins install --target ./target.json --path .  --root ./DragonReach

build-linux:
	cargo -Z build-std=core,alloc,compiler_builtins build --target x86_64-unknown-linux-gnu