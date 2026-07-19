SHELL := /bin/zsh

OUTPUT_DIR := dmg/app
RELEASE_DIR := releases/v0.1.0
BUNDLE_DIR := src-tauri/target/release/bundle
RUST_PATH := $(HOME)/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$(PATH)

.PHONY: export frontend bundle dmg app pkg clean

# `make export` is the one-shot release command. It always rebuilds the web
# frontend first, then creates all native bundles from that same build.
export: bundle
	mkdir -p "$(OUTPUT_DIR)"
	rm -rf "$(OUTPUT_DIR)/Codex Meters.app"
	rm -f "$(OUTPUT_DIR)"/*.dmg(N) "$(OUTPUT_DIR)/Codex Meters.pkg"
	cp "$(BUNDLE_DIR)"/dmg/*.dmg "$(OUTPUT_DIR)/"
	ditto "$(BUNDLE_DIR)/macos/Codex Meters.app" "$(OUTPUT_DIR)/Codex Meters.app"
	pkgbuild --component "$(BUNDLE_DIR)/macos/Codex Meters.app" --install-location "/Applications" --scripts "scripts/pkg" --identifier "com.codex.tokenmeter" --version "0.1.0" "$(OUTPUT_DIR)/Codex Meters.pkg"
	mkdir -p "$(RELEASE_DIR)"
	cp "$(OUTPUT_DIR)/Codex Meters.pkg" "$(RELEASE_DIR)/Codex Meters.pkg"
	cp "$(OUTPUT_DIR)"/*.dmg "$(RELEASE_DIR)/"

frontend:
	bun run build

bundle: frontend
	mkdir -p "$(OUTPUT_DIR)"
	rm -rf "$(BUNDLE_DIR)/dmg" "$(BUNDLE_DIR)/macos/Codex Meters.app"
	PATH="$(RUST_PATH)" bun run tauri build --bundles app dmg

dmg: bundle
	rm -f "$(OUTPUT_DIR)"/*.dmg(N)
	cp "$(BUNDLE_DIR)"/dmg/*.dmg "$(OUTPUT_DIR)/"

app: bundle
	rm -rf "$(OUTPUT_DIR)/Codex Meters.app"
	ditto "$(BUNDLE_DIR)/macos/Codex Meters.app" "$(OUTPUT_DIR)/Codex Meters.app"

pkg: bundle
	rm -f "$(OUTPUT_DIR)/Codex Meters.pkg"
	pkgbuild --component "$(BUNDLE_DIR)/macos/Codex Meters.app" --install-location "/Applications" --scripts "scripts/pkg" --identifier "com.codex.tokenmeter" --version "0.1.0" "$(OUTPUT_DIR)/Codex Meters.pkg"

clean:
	rm -rf "$(OUTPUT_DIR)"
