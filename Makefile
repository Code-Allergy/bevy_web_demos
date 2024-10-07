# Ryan Schaffer | rys686 | 11295237
# CMPT 485 - Presentation Demos
# Makefile for building the website demos

# Dwight would not be happy with this... I'm too lazy to make this work.
DIST_DIR := dist
BIND_DIR_NAME := binds
WASM_TMP_DIR := $(DIST_DIR)/wasm
ASSETS_OUT_DIR := $(DIST_DIR)/assets
JS_OUT_NAME := bind
BINDINGS_OUT_DIR := $(DIST_DIR)/$(BIND_DIR_NAME)
CODE_OUT_DIR := $(DIST_DIR)/code
MODULES_LIST := $(DIST_DIR)/modules.txt

RUST_SRC_FILES := $(shell find src -name "*.rs")

WASM_GLOB := target/wasm32-unknown-unknown/debug/*.wasm
CARGO_FLAGS :=

# Uncomment to build in release mode
# WASM_GLOB := target/wasm32-unknown-unknown/release/*.wasm
# CARGO_FLAGS := --release


all: build copy_files create_bindings generate_modules

serve: all
	@echo "Starting server..."
	cd $(DIST_DIR) && python3 -m http.server

build: $(RUST_SRC_FILES)
	@echo "Building..."
	cargo build --all-targets --target wasm32-unknown-unknown $(CARGO_FLAGS)

# Copy the wasm files to the assets folder
copy_files: build
# 	@echo "Copying wasm files to $(ASSETS_OUT_DIR)"
	mkdir -p $(WASM_TMP_DIR)
	cp $(WASM_GLOB) $(WASM_TMP_DIR)

	# Copy assets to the dist folder
	@echo "Copying assets to $(ASSETS_OUT_DIR)"
	mkdir -p $(ASSETS_OUT_DIR)
	cp -r assets/* $(ASSETS_OUT_DIR)

	# Copy source files to the dist folder
	@echo "Copying source files to $(CODE_OUT_DIR)"
	mkdir -p $(CODE_OUT_DIR)
	cp -r src/* $(CODE_OUT_DIR)
	
	@echo "Copying index.html to $(DIST_DIR)"
	cp index.html $(DIST_DIR)

	@echo "Copying loader to $(DIST_DIR)"
	cp loader.js $(DIST_DIR)

	@echo "Copying style.css to $(DIST_DIR)"
	cp style.css $(DIST_DIR)

# Create bindings for each wasm file
create_bindings: copy_files
	@echo "Creating bindings..."
	@mkdir -p $(BINDINGS_OUT_DIR)
	find $(WASM_TMP_DIR) -type f -name "*.wasm" | while read -r wasm_file; do \
		wasm_file_stripped=$$(basename $$wasm_file .wasm); \
		echo "Creating bindings for $$wasm_file"; \
		echo "Will bind at $(BINDINGS_OUT_DIR)/$$wasm_file_stripped"; \
		mkdir -p $(BINDINGS_OUT_DIR)/$$wasm_file_stripped; \
		wasm-bindgen --no-typescript --target web --out-dir $(BINDINGS_OUT_DIR)/$$wasm_file_stripped --out-name $(JS_OUT_NAME) $$wasm_file; \
	done
	rm -rf $(WASM_TMP_DIR)

# Generate the modules list
generate_modules: create_bindings
	@echo "Generating modules list..."
	find $(BINDINGS_OUT_DIR) -name "bind.js" | sed 's|^$(DIST_DIR)/||' | sed 's|^|./|' | sort > $(MODULES_LIST)


# Clean target
clean:
	@echo "Cleaning up..."
	rm -rf $(DIST_DIR)/*
	cargo clean

.PHONY: all copy_files create_bindings generate_modules clean serve