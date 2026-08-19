.PHONY: build release install uninstall test clean help

BINARY := kumo
INSTALL_DIR := /usr/local/bin
VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')

## help: Show this help message
help:
	@echo "☁  kumo (雲) — v$(VERSION)"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Development:"
	@echo "  build        Build debug binary"
	@echo "  release      Build optimized release binary"
	@echo "  test         Run tests"
	@echo "  clean        Remove build artifacts"
	@echo ""
	@echo "Installation:"
	@echo "  install      Build release and install to $(INSTALL_DIR)"
	@echo "  uninstall    Remove kumo from $(INSTALL_DIR)"
	@echo ""
	@echo "Release:"
	@echo "  dist         Build release binaries for distribution"
	@echo "  checksum     Generate SHA256 checksums"
	@echo ""

## build: Build debug binary
build:
	cargo build

## release: Build optimized release binary
release:
	cargo build --release
	@echo ""
	@echo "Binary: target/release/$(BINARY)"
	@ls -lh target/release/$(BINARY)

## test: Run tests
test:
	cargo test

## install: Build release and install to /usr/local/bin
install: release
	@echo "Installing $(BINARY) to $(INSTALL_DIR)..."
	@if [ -w "$(INSTALL_DIR)" ]; then \
		rm -f $(INSTALL_DIR)/$(BINARY); \
		cp target/release/$(BINARY) $(INSTALL_DIR)/$(BINARY); \
	else \
		sudo rm -f $(INSTALL_DIR)/$(BINARY); \
		sudo cp target/release/$(BINARY) $(INSTALL_DIR)/$(BINARY); \
	fi
	@chmod +x $(INSTALL_DIR)/$(BINARY)
	@echo "✓ $(BINARY) v$(VERSION) installed to $(INSTALL_DIR)/$(BINARY)"

## uninstall: Remove kumo from /usr/local/bin
uninstall:
	@echo "Removing $(BINARY) from $(INSTALL_DIR)..."
	@if [ -w "$(INSTALL_DIR)" ]; then \
		rm -f $(INSTALL_DIR)/$(BINARY); \
	else \
		sudo rm -f $(INSTALL_DIR)/$(BINARY); \
	fi
	@rm -rf ~/.kumo
	@echo "✓ $(BINARY) uninstalled"

## clean: Remove build artifacts
clean:
	cargo clean

## dist: Build release binaries for all platforms (requires cross-compilation)
dist: release
	@mkdir -p dist
	@cp target/release/$(BINARY) dist/$(BINARY)
	@cd dist && tar -czf $(BINARY)-$(shell uname -s | tr '[:upper:]' '[:lower:]')-$(shell uname -m).tar.gz $(BINARY)
	@rm dist/$(BINARY)
	@echo "Distribution package: dist/"
	@ls -lh dist/

## checksum: Generate SHA256 checksums for dist files
checksum:
	@cd dist && shasum -a 256 *.tar.gz > checksums.txt
	@cat dist/checksums.txt
