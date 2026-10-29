VERSION := $(shell awk -F '"' '/^version = / { print $$2; exit }' Cargo.toml)
ARCH := $(shell uname -m)
ARCHIVE := Sleep-Monitors-v$(VERSION)-macos-$(ARCH).zip

.PHONY: all run release check

all:
	cargo build --release --locked --target-dir target
	mkdir -p "dist/Sleep Monitors.app/Contents/MacOS"
	cp target/release/macos-sleep-monitors "dist/Sleep Monitors.app/Contents/MacOS/"
	cp Info.plist "dist/Sleep Monitors.app/Contents/"
	codesign --force --sign - "dist/Sleep Monitors.app"

run: all
	open "dist/Sleep Monitors.app"

release: all
	ditto -c -k --sequesterRsrc --keepParent "dist/Sleep Monitors.app" "dist/$(ARCHIVE)"
	cd dist && shasum -a 256 "$(ARCHIVE)" > "$(ARCHIVE).sha256"

check:
	cargo fmt --check
	cargo clippy --locked --all-targets -- -D warnings
	plutil -lint Info.plist
