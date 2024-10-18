# Ryan Schaffer | rys686 | 11295237
# CMPT 485 - Presentation Demos
# Makefile for building the website demos

DIST_DIR := dist
BIND_DIR_NAME := binds
WASM_TMP_DIR := $(DIST_DIR)/wasm
ASSETS_OUT_DIR := $(DIST_DIR)/assets
JS_OUT_NAME := bind
BINDINGS_OUT_DIR := $(DIST_DIR)/$(BIND_DIR_NAME)
CODE_OUT_DIR := $(DIST_DIR)/code
MODULES_LIST := $(DIST_DIR)/modules.txt

RUST_SRC_FILES := $(shell find src -name "*.rs")

# Default values
RELEASE := false

# Set the WASM_BUILD_DIR and CARGO_FLAGS based on the RELEASE flag
ifeq ($(RELEASE), true)
    WASM_BUILD_DIR := target/wasm32-unknown-unknown/release
    CARGO_FLAGS := --release
else
    WASM_BUILD_DIR := target/wasm32-unknown-unknown/debug
    CARGO_FLAGS :=
endif

WASM_OUTPUT_FILES := $(shell find $(WASM_BUILD_DIR) -maxdepth 1 -type f -name "*.wasm")
ASSETS := $(shell find assets -type f)

BINDING_FILES := $(patsubst $(WASM_BUILD_DIR)/%.wasm,$(BINDINGS_OUT_DIR)/%/$(JS_OUT_NAME).js,$(WASM_OUTPUT_FILES))

DIST_FILES := $(DIST_DIR)/index.html $(DIST_DIR)/loader.js $(DIST_DIR)/style.css
DIST_ASSETS := $(patsubst assets/%,$(ASSETS_OUT_DIR)/%,$(ASSETS))

LAST_BUILD := target/last_build.timestamp

.PHONY: all serve build copy_files create_bindings generate_modules clean-cargo clean-dist clean

all: build copy_files create_bindings $(MODULES_LIST)

serve: all
	@echo "Starting server..."
	cd $(DIST_DIR) && python3 -m http.server

# Build only if Rust source files have changed or if the last build file is missing
build: $(LAST_BUILD)

$(LAST_BUILD): $(RUST_SRC_FILES)
	@echo "Building..."
	cargo build --all-targets --target wasm32-unknown-unknown $(CARGO_FLAGS)
	@touch $@  # Update the timestamp


# COPY ALL STATIC FILES TO THE DIST DIRECTORY
copy_files: $(DIST_FILES) $(DIST_ASSETS)

$(DIST_DIR)/%: % | $(DIST_DIR)
	@echo "Copying $< to $(DIST_DIR)"
	cp $< $@

$(ASSETS_OUT_DIR)/%: assets/% | $(ASSETS_OUT_DIR)
	@echo "Copying asset $< to $(ASSETS_OUT_DIR)"
	@mkdir -p $(dir $@)
	cp $< $@

$(DIST_DIR) $(ASSETS_OUT_DIR):
	mkdir -p $@

#
# CREATE BINDINGS FOR WEBASSEMBLY FILES
#
create_bindings: $(BINDING_FILES) copy_files

$(BINDINGS_OUT_DIR)/%/$(JS_OUT_NAME).js: $(WASM_BUILD_DIR)/%.wasm
	@echo "Creating bindings for $<"
	@mkdir -p $(dir $@)  # Ensure the output directory exists
	wasm-bindgen --no-typescript --target web --out-dir $(BINDINGS_OUT_DIR)/$(basename $*) --out-name $(JS_OUT_NAME) $<
	
	@if [ "$(RELEASE)" = "true" ]; then \
		echo "Optimizing WebAssembly file with wasm-opt..."; \
		wasm-opt -Oz $(BINDINGS_OUT_DIR)/$(basename $*)/$(JS_OUT_NAME)_bg.wasm -o $(BINDINGS_OUT_DIR)/$(basename $*)/$(JS_OUT_NAME)_bg.wasm; \
	fi

#
# GENERATE A LIST OF WASM MODULE BINDS
#
$(MODULES_LIST): $(BINDING_FILES)
	@echo "Generating modules list..."
	find $(BINDINGS_OUT_DIR) -name "bind.js" | sed 's|^$(DIST_DIR)/||' | sed 's|^|./|' | sort > $(MODULES_LIST)

#
# CLEAN UP
#
clean: clean-cargo clean-dist

clean-cargo:
	@echo "Cleaning up cargo..."
	cargo clean

clean-dist:
	@echo "Cleaning up dist..."
	rm -rf $(DIST_DIR)/*
