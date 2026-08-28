PREFIX ?= /usr/local
DESTDIR ?=
CARGO ?= cargo

WAYCAT_VERSION := $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)

.PHONY: all build install uninstall clean

all: build

build:
	$(CARGO) build --release

install: build
	install -Dm755 target/release/waycat $(DESTDIR)$(PREFIX)/bin/waycat
	install -Dm644 LICENSE $(DESTDIR)$(PREFIX)/share/licenses/waycat/LICENSE
	install -Dm644 res/waycat.ttf $(DESTDIR)$(PREFIX)/share/fonts/TTF/waycat.ttf

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/waycat \
		$(DESTDIR)$(PREFIX)/share/licenses/waycat/LICENSE \
		$(DESTDIR)$(PREFIX)/share/fonts/TTF/waycat.ttf

clean:
	$(CARGO) clean