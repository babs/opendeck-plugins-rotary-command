TARGET ?= x86_64-unknown-linux-gnu
OUTPUT ?= dist
BINARY = opendeck-rotary-generic
UUID = info.degois.damien.opendeck.plugins.rotary-command
PLUGIN_DIR = $(UUID).sdPlugin

.PHONY: build clean install package

build:
	rm -rf $(OUTPUT)
	cp -r assets $(OUTPUT)
	@cargo build --release --target $(TARGET)
	cp target/$(TARGET)/release/$(BINARY) $(OUTPUT)/$(BINARY)-$(TARGET)

clean:
	rm -rf $(OUTPUT) $(UUID).zip
	cargo clean

install: build
	mkdir -p $(HOME)/.config/opendeck/plugins/$(PLUGIN_DIR)
	cp -r $(OUTPUT)/* $(HOME)/.config/opendeck/plugins/$(PLUGIN_DIR)/

package: build
	rm -rf $(PLUGIN_DIR) $(UUID).zip
	cp -r $(OUTPUT) $(PLUGIN_DIR)
	@zip -qr $(UUID).zip $(PLUGIN_DIR)
	rm -rf $(PLUGIN_DIR)
