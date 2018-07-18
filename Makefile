# This is a DEVELOPMENT makefile, used to quickly run the whole thing in development.

PROFILE ?= debug
BUILDWRAP ?= nice -n20 time
CARGO ?= cargo +nightly
RUST_BACKTRACE ?= full
RUST_LOG ?= info
RUSTFLAGS ?= -Zexternal-macro-backtrace
# RUSTFLAGS ?= -Zcross-lang-lto -Zexternal-macro-backtrace # broke lld recently: "PlaceholderQueue hasn't been flushed"

.PHONY: run

SRC_COMPOSITOR != find compositor -name '*.rs'
SRC_PROTO != find protos -name '*.rs'
SRC_SHELL != find shell -name '*.rs'
SRC_LOGINW != find loginw -name '*.rs'
SRC_WESTON != find weston-rs -name '*.rs'

target/debug/dankshell-compositor: $(SRC_COMPOSITOR) $(SRC_PROTO) $(SRC_LOGINW) $(SRC_WESTON)
	cd compositor && RUSTFLAGS="$(RUSTFLAGS)" $(BUILDWRAP) $(CARGO) build

target/debug/dankshell-shell-experience: $(SRC_SHELL) $(SRC_PROTO)
	cd shell && RUSTFLAGS="$(RUSTFLAGS)" $(BUILDWRAP) $(CARGO) build

target/release/dankshell-compositor: $(SRC_COMPOSITOR) $(SRC_PROTO) $(SRC_LOGINW) $(SRC_WESTON)
	cd compositor && RUSTFLAGS="$(RUSTFLAGS)" $(BUILDWRAP) $(CARGO) build --release

target/release/dankshell-shell-experience: $(SRC_SHELL) $(SRC_PROTO)
	cd shell && RUSTFLAGS="$(RUSTFLAGS)" $(BUILDWRAP) $(CARGO) build --release

run: target/$(PROFILE)/dankshell-compositor target/$(PROFILE)/dankshell-shell-experience
	RUST_BACKTRACE="$(RUST_BACKTRACE)" RUST_LOG="$(RUST_LOG)" \
		PATH="./target/$(PROFILE):$$PATH" \
		target/$(PROFILE)/dankshell-compositor
