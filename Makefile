SHELL := /bin/zsh

OUTPUT_DIR := dmg/app
RELEASE_DIR := releases/v0.1.0
BUNDLE_DIR := src-tauri/target/release/bundle
STAGING_ROOT := $(OUTPUT_DIR)/.pkg-root
PKG_PATH := $(OUTPUT_DIR)/Codex Meters.pkg
RUST_PATH := $(HOME)/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$(PATH)
PACKAGE_COMPONENTS := scripts/pkg-components.plist
PACKAGED_APP_PATH := Codex Meters.app

.PHONY: export frontend bundle dmg app pkg stage-pkg clean

# `make export` is the one-shot release command. It always rebuilds the web
# frontend first, then creates all native bundles from that same build.
export: bundle
	mkdir -p "$(OUTPUT_DIR)"
	rm -rf "$(OUTPUT_DIR)/$(PACKAGED_APP_PATH)"
	rm -f "$(OUTPUT_DIR)"/*.dmg(.N) "$(PKG_PATH)"
	rm -rf "$(STAGING_ROOT)"
	cp "$(BUNDLE_DIR)"/dmg/*.dmg "$(OUTPUT_DIR)/"
	ditto "$(BUNDLE_DIR)/macos/Codex Meters.app" "$(OUTPUT_DIR)/Codex Meters.app"
	$(MAKE) pkg
	./scripts/verify-pkg.sh "$(PKG_PATH)"
	mkdir -p "$(RELEASE_DIR)"
	cp "$(PKG_PATH)" "$(RELEASE_DIR)/Codex Meters.pkg"
	cp "$(OUTPUT_DIR)"/*.dmg "$(RELEASE_DIR)/"

frontend:
	bun run build

bundle: frontend
	mkdir -p "$(OUTPUT_DIR)"
	rm -rf "$(BUNDLE_DIR)/dmg" "$(BUNDLE_DIR)/macos/Codex Meters.app"
	PATH="$(RUST_PATH)" bun run tauri build --bundles app dmg

dmg: bundle
	rm -f "$(OUTPUT_DIR)"/*.dmg(.N)
	cp "$(BUNDLE_DIR)"/dmg/*.dmg "$(OUTPUT_DIR)/"

app: bundle
	rm -rf "$(OUTPUT_DIR)/$(PACKAGED_APP_PATH)"
	ditto "$(BUNDLE_DIR)/macos/Codex Meters.app" "$(OUTPUT_DIR)/Codex Meters.app"

pkg: stage-pkg
	pkgbuild --root "$(STAGING_ROOT)" --component-plist "$(PACKAGE_COMPONENTS)" --install-location / --scripts "scripts/pkg" --identifier "com.codex.tokenmeter" --version "0.1.0" "$(PKG_PATH)"
	PKG_VERIFY_ROOT="$$(mktemp -d)" ; \
	PKG_VERIFY_DIR="$$PKG_VERIFY_ROOT/expanded" ; \
	/usr/sbin/pkgutil --expand "$(PKG_PATH)" "$$PKG_VERIFY_DIR" && \
	/usr/bin/perl -0pi -e "s/\\n\\s*<relocate\\/>\\n//g; s/\\n\\s*<relocate>.*?<\\/relocate>\\n//gs" "$$PKG_VERIFY_DIR/PackageInfo" && \
	/usr/sbin/pkgutil --flatten "$$PKG_VERIFY_DIR" "$(PKG_PATH)" ; \
	rm -rf "$$PKG_VERIFY_ROOT"
	rm -rf "$(STAGING_ROOT)"

stage-pkg: bundle
	rm -rf "$(STAGING_ROOT)"
	mkdir -p "$(STAGING_ROOT)/Applications"
	ditto "$(BUNDLE_DIR)/macos/Codex Meters.app" "$(STAGING_ROOT)/Applications/Codex Meters.app"

clean:
	rm -rf "$(OUTPUT_DIR)"
