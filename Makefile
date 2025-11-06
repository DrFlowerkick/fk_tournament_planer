# -------- Constants & Configuration --------

CARGO_LEPTOS := cargo leptos

# -------- App Configuration --------
SERVER_NAME := fk_tournament_planer
WEB_PORT := 3000

# -------- Code Formatting --------

.PHONY: fmt
fmt:
	cargo fmt --all && git ls-files '*.rs' | xargs -r -n 200 leptosfmt

# -------- Leptos clippy --------

.PHONY: clippy
clippy:
	cargo clippy --all-targets --all-features

# -------- Leptos lint: fmt + clippy --------

.PHONY: lint
lint:
	cargo fmt --all && git ls-files '*.rs' | xargs -r -n 200 leptosfmt && cargo clippy --all-targets --all-features

# -------- Cleanup --------

.PHONY: clean
clean:
	cargo clean

# --- E2E with temp DB ---------------------------------------------------------
# loading .env
ifneq ("$(wildcard .env)","")
  include .env
  export
endif

POSTGRES_URL_BASE := $(patsubst %/,%,$(POSTGRES_URL))          # trim trailing /
PSQL_URL          := $(POSTGRES_URL_BASE)/postgres
# generate DB name once
ifndef E2E_DB_NAME
  E2E_DB_NAME := tst_$(shell (uuidgen 2>/dev/null || cat /proc/sys/kernel/random/uuid) | tr 'A-Z' 'a-z' | tr -d -)
endif
export E2E_DB_NAME
E2E_DB_URL        := $(POSTGRES_URL_BASE)/$(E2E_DB_NAME)

# Quiet default logs for E2E; override with: make e2e E2E_LOG=info,app=info
E2E_LOG ?= warn,server=warn,app=warn,app_core=warn,db_postgres=warn,leptos_axum=warn,tower_http=error,hyper=error,diesel=error

.PHONY: e2e e2e_pre e2e_drop

e2e_pre:
	@echo "🧪 create temp DB: $(E2E_DB_NAME)"
	psql "$(PSQL_URL)" -v ON_ERROR_STOP=1 -c 'CREATE DATABASE "$(E2E_DB_NAME)" TEMPLATE template0;'
	@if command -v diesel >/dev/null 2>&1; then \
	  echo "🔧 migrate (diesel) on $(E2E_DB_NAME)"; \
	  DATABASE_URL="$(E2E_DB_URL)" diesel migration run --migration-dir ./db_postgres/migrations; \
	else \
	  echo "ℹ️ diesel not found: skip CLI migration (ensure app migrates on start)"; \
	fi

e2e_drop:
	@echo "🧹 drop temp DB: $(E2E_DB_NAME)"
	-psql "$(PSQL_URL)" -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname='$(E2E_DB_NAME)';" >/dev/null
	-psql "$(PSQL_URL)" -c 'DROP DATABASE IF EXISTS "$(E2E_DB_NAME)";' >/dev/null

# main e2e target: create DB → set ENV → start E2E → drop DB (even if error)
e2e: e2e_pre
	@set -e; \
	echo "▶ run cargo leptos end-to-end against $(E2E_DB_NAME)"; \
	\
	# Pass all env to the single cargo process
	env \
	  RUST_LOG="$(E2E_LOG)" \
	  DATABASE_NAME="$(E2E_DB_NAME)" \
	  cargo leptos end-to-end --release \
	|| { rc=$$?; $(MAKE) -s e2e_drop E2E_DB_NAME=$(E2E_DB_NAME); exit $$rc; }
	@$(MAKE) -s e2e_drop E2E_DB_NAME=$(E2E_DB_NAME)

# -------- SSR Build & Run --------

.PHONY: dev-ssr
dev-ssr:
	$(CARGO_LEPTOS) watch | bunyan

.PHONY: build-ssr
build-ssr:
	$(CARGO_LEPTOS) build --release

.PHONY: run-ssr
run-ssr:
	$(CARGO_LEPTOS) serve --release

# -------- Unit Testing & Coverage --------
.PHONY: test
test:
	cargo nextest run --workspace --features "ssr"
	cargo test --doc --workspace 

.PHONY: test-release
test-release:
	cargo nextest run --workspace --locked --release --features "ssr"

.PHONY: coverage
coverage:
	cargo llvm-cov nextest --workspace --locked --features "ssr" --lcov --output-path coverage/lcov.info
	cargo llvm-cov report --release --html --output-dir coverage

.PHONY: coverage-release
coverage-release:
	cargo llvm-cov nextest --workspace --locked --release --features "ssr" --lcov --output-path coverage/lcov.info
	cargo llvm-cov report --release --html --output-dir coverage

# -------- Webserver Monitoring & Control --------

.PHONY: webserver
webserver:
	@lsof -i :$(WEB_PORT)

.PHONY: kill-webserver
kill-webserver:
	@echo "🔍 Checking for listeners on port $(WEB_PORT)..."
	@PIDS=$$(lsof -ti TCP:$(WEB_PORT) -sTCP:LISTEN); \
	if [ -n "$$PIDS" ]; then \
		echo "🛑 Found PID(s): $$PIDS — sending SIGTERM"; \
		kill $$PIDS || true; \
		sleep 1; \
		PIDS2=$$(lsof -ti TCP:$(WEB_PORT) -sTCP:LISTEN); \
		if [ -n "$$PIDS2" ]; then \
			echo "⛔ Still running: $$PIDS2 — sending SIGKILL"; \
			kill -9 $$PIDS2 || true; \
		else \
			echo "✅ Port $(WEB_PORT) is free now."; \
		fi \
	else \
		echo "✅ No listener on port $(WEB_PORT)."; \
	fi

# Kill process group of the first listener on WEB_PORT
.PHONY: kill-webserver-pg
kill-webserver-pg:
	@PID=$$(lsof -ti TCP:$(WEB_PORT) -sTCP:LISTEN | head -n1); \
	if [ -n "$$PID" ]; then \
		PGID=$$(ps -o pgid= $$PID | tr -d ' '); \
		echo "🛑 Killing process group $$PGID (via -$$PGID)"; \
		kill -TERM -$$PGID || true; \
	else \
		echo "✅ No listener on port $(WEB_PORT)."; \
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
