.PHONY: all build clean test start dev

all: build test

build:
	cd src-tauri/ && cargo build

dev: 
	npm run tauri dev

test:
	cd src-tauri/ && cargo test