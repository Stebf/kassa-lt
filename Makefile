.PHONY: all build clean test dev release

all: build test

build:
	cd src-tauri/ && cargo build

dev: 
	npm run tauri dev

test:
	cd src-tauri/ && cargo test

release-windows:
	npm run tauri build -- --runner cargo-xwin --target x86_64-pc-windows-msvc
	cp src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/* .
	echo "Release built successfully. The installer is located in the current directory."

release-android:
	npm run tauri android build -- --apk

release-macos:
	npm run tauri build -- --bundles app