MAKEJS_FILE := Makefile.mjs
MAKEJS := node $(CFLAGS) $(MAKEJS_FILE)

.PHONY: build

define verify_makejs_path =
	@if ! [ -f $(MAKEJS_FILE) ]; then \
		./scripts/setup_make.sh; \
	fi
endef

build:
	$(call verify_makejs_path)
	$(MAKEJS) build $(ARGS)
