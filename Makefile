BIN_NAME := sora-on-rust
TARGET ?= x86_64-unknown-linux-gnu
BUILD_TYPE ?= stable

ifeq ($(BUILD_TYPE),stable)
CARGO_FLAGS := --features stable
PROFILE := release
endif

ifeq ($(BUILD_TYPE),performance)
CARGO_FLAGS := --no-default-features --features performance
PROFILE := release
endif

ifeq ($(BUILD_TYPE),profiling)
CARGO_FLAGS := --no-default-features --features profiling
PROFILE := profiling
endif

OUT_DIR := target/$(TARGET)/$(PROFILE)
OUT_BIN := $(OUT_DIR)/$(BIN_NAME)
DIST_BIN := $(BIN_NAME)-$(BUILD_TYPE)-$(TARGET)

.PHONY: all build install-cross rename clean

all: build rename

install-cross:
	cargo install cross --git https://github.com/cross-rs/cross

build:
	cross build --profile $(PROFILE) --target $(TARGET) $(CARGO_FLAGS)

rename:
	mv $(OUT_BIN) $(DIST_BIN)

clean:
	rm -f $(BIN_NAME)-*-*
