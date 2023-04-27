sources-utils := .cargo/config.toml Cargo.toml $(shell find . -path "./src/*.rs" -o -path "./src/*/*.rs")
sources-macros := macros/Cargo.toml $(shell find macros -path "macros/src/*.rs")

utils: build-default
build: build-default
build-default: $(sources-utils) $(sources-macros)
	@cargo build-default
build-optimized: $(sources-utils) $(sources-macros)
	@cargo build-optimized
build-debug: $(sources-utils) $(sources-macros)
	@cargo build-debug
build-asm: $(sources-utils) $(sources-macros)
	@cargo build-asm
build-ir: $(sources-utils) $(sources-macros)
	@cargo build-ir
build-bc: $(sources-utils) $(sources-macros)
	@cargo build-bc
build-mir: $(sources-utils) $(sources-macros)
	@cargo build-mir
syntax: $(sources-utils) $(sources-macros)
	@cargo syntax
expand: $(sources-utils) $(sources-macros)
	@cargo macro-expand > cargo_expand.rs
backtrace: $(sources-utils) $(sources-macros)
	@cargo +nightly macro-backtrace

macros: macros-default
macros-default: $(sources-macros)
	@cd macros && cargo build-default
macros-optimized: $(sources-macros)
	@cd macros && cargo build-optimized
macros-debug: $(sources-macros)
	@cd macros && cargo build-debug
macros-asm: $(sources-macros)
	@cd macros && cargo build-asm
macros-ir: $(sources-macros)
	@cd macros && cargo build-ir
macros-bc: $(sources-macros)
	@cd macros && cargo build-bc
macros-mir: $(sources-macros)
	@cd macros && cargo build-mir
macros-syntax: $(sources-macros)
	@cd macros && cargo syntax
macros-expand: $(sources-macros)
	@cd macros && cargo macro-expand > cargo_expand.rs
macros-backtrace: $(sources-macros)
	@cd macros && cargo +nightly macro-backtrace
