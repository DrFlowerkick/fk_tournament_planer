# ---------- Stage 1: Build ----------
FROM rust:bookworm AS builder

# Install system dependencies
RUN apt-get update -y && apt-get install -y --no-install-recommends clang lld npm wget ca-certificates && apt-get clean -y && rm -rf /var/lib/apt/lists/*

ENV CARGO_TERM_COLOR=always

# Install cargo-binstall, which makes it easier to install other
# cargo extensions like cargo-leptos
RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN cp cargo-binstall /usr/local/cargo/bin

# get github token from publish.yml and install cargo-leptos
RUN --mount=type=secret,id=github_token GITHUB_TOKEN=$(cat /run/secrets/github_token) cargo binstall cargo-leptos -y

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

# Set up workdir and copy source
WORKDIR /app
COPY . .

# Install frontend dependencies (for Tailwind, daisyUI, etc.)
# Assumes package.json/package-lock.json in project root
RUN npm ci

# Set the Leptos WASM Bindgen version via build argument
ARG LEPTOS_WASM_BINDGEN_VERSION=0.2.105
ENV LEPTOS_WASM_BINDGEN_VERSION=$LEPTOS_WASM_BINDGEN_VERSION

# Build the Leptos app (WASM + SSR) in release mode
RUN cargo leptos build --release

# ---------- Stage 2: Runtime ----------
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies (for OpenSSL etc.)
RUN apt-get update -y && apt-get install -y --no-install-recommends openssl ca-certificates && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy runtime files from build container
COPY --from=builder /app/target/release/server /app/fk_tournament_planer
COPY --from=builder /app/target/site /app/site
#COPY --from=builder /app/Cargo.toml /app/

# Set Leptos runtime environment variables (can be overridden!)
#ENV RUST_LOG="info"
ENV RUST_LOG="info,server=info,app=info,app_core=info,db_postgres=debug,tower_http=warn,hyper=warn,diesel=debug"
ENV LEPTOS_ENV="PROD"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_OUTPUT_NAME="fk_tournament_planer"
ENV DATABASE_NAME="fk_tournament_planer"

# Expose SSR port
EXPOSE 8080

# Start the Leptos SSR server
CMD ["/app/fk_tournament_planer"]
