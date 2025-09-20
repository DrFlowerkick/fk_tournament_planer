# -------- Constants & Configuration --------

CARGO_LEPTOS := cargo leptos

# use this flags when developing for faster compile
DEV_RUSTFLAGS := RUSTFLAGS="--cfg erase_components"

# -------- App Configuration --------
SERVER_NAME := fk_tournament_planer
WEB_PORT := 3000

# -------- Code Formatting --------

.PHONY: fmt
fmt:
	cargo fmt && leptosfmt ./**/*.rs

# -------- Leptos clippy --------

.PHONY: clippy
clippy:
  cargo clippy

# -------- Leptos lint: fmt + clippy --------

.PHONY: lint
lint:
  leptosfmt ./**/*.rs && cargo fmt && cargo clippy

# -------- Cleanup --------

.PHONY: clean
clean:
	cargo clean

# -------- SSR Build, E2E Test & & Run --------

.PHONY: dev-ssr
dev-ssr:
	$(DEV_RUSTFLAGS) $(CARGO_LEPTOS) watch

.PHONY: e2e-ssr
e2e-ssr:
	$(CARGO_LEPTOS) end-to-end --release

.PHONY: build-ssr
build-ssr:
	$(CARGO_LEPTOS) build --release

.PHONY: run-ssr
run-ssr:
	$(CARGO_LEPTOS) serve --release

# -------- Webserver Monitoring & Control --------

.PHONY: webserver
webserver:
	@lsof -i :$(WEB_PORT)

.PHONY: kill-webserver
kill-webserver:
	@echo "🔍 Checking for running $(SERVER_NAME) server on port $(WEB_PORT)..."
	@PID=$$(lsof -i :$(WEB_PORT) -sTCP:LISTEN -t -a -c $(SERVER_NAME)); \
	if [ -n "$$PID" ]; then \
		echo "🛑 Found $(SERVER_NAME) (PID: $$PID), stopping it..."; \
		kill $$PID; \
	else \
		echo "✅ No $(SERVER_NAME) server running on port $(WEB_PORT)."; \
	fi

# -------- Set release tag to build docker on github --------
# only use this, if you do not use release-please

.PHONY: release-tag
release-tag:
	@echo "🔍 Lese Version aus Cargo.toml..."
	@VERSION=$$(grep '^version =' Cargo.toml | sed -E 's/version = "(.*)"/\1/') && \
	TAG="v$$VERSION" && \
	echo "🏷  Erzeuge Git-Tag: $$TAG" && \
	if git rev-parse "$$TAG" >/dev/null 2>&1; then \
		echo "❌ Tag '$$TAG' existiert bereits. Abbruch."; \
		exit 1; \
	fi && \
	git tag "$$TAG" && \
	git push origin "$$TAG" && \
	echo "✅ Git-Tag '$$TAG' erfolgreich erstellt und gepusht."
