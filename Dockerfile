# ForgeFabrik — multi-stage Docker build
#
# Stage 1: build the release binary
# Stage 2: minimal runtime image (no Rust toolchain)
#
# Build:  docker build -t forgefabrik .
# Run:    docker run -p 8080:8080 forgefabrik

# ── Stage 1: builder ──────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install system deps needed by crates (ring → gcc, openssl)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies before copying source
# This layer is rebuilt only when Cargo.toml / Cargo.lock change.
COPY Cargo.toml Cargo.lock ./
COPY foundation/types/Cargo.toml        foundation/types/
COPY foundation/contracts/Cargo.toml    foundation/contracts/
COPY domain/world/Cargo.toml            domain/world/
COPY domain/agents/Cargo.toml           domain/agents/
COPY domain/economy/Cargo.toml          domain/economy/
COPY domain/quests/Cargo.toml           domain/quests/
COPY domain/security/Cargo.toml         domain/security/
COPY domain/consensus/Cargo.toml        domain/consensus/
COPY runtime/drivers/Cargo.toml         runtime/drivers/
COPY runtime/sandbox/Cargo.toml         runtime/sandbox/
COPY runtime/plugin/Cargo.toml          runtime/plugin/
COPY runtime/server/Cargo.toml          runtime/server/
COPY runtime/cli/Cargo.toml             runtime/cli/
COPY plugins/plugin-agents/Cargo.toml   plugins/plugin-agents/
COPY plugins/plugin-world/Cargo.toml    plugins/plugin-world/
COPY plugins/plugin-gm/Cargo.toml       plugins/plugin-gm/
COPY plugins/plugin-economy/Cargo.toml  plugins/plugin-economy/

# Create stub lib/main files so `cargo build` can resolve the dependency graph
# without the full source tree.
RUN find . -name "Cargo.toml" ! -path "./Cargo.toml" | while read f; do \
      dir=$(dirname "$f"); \
      mkdir -p "$dir/src"; \
      if grep -q '\[\[bin\]\]' "$f" || grep -q 'name = "cli"' "$f"; then \
        echo 'fn main(){}' > "$dir/src/main.rs"; \
      else \
        echo '' > "$dir/src/lib.rs"; \
      fi; \
    done

RUN cargo build --release -p cli 2>&1 | tail -5

# Now copy real source and rebuild only what changed
COPY . .
# Touch lib/main files so Cargo knows source changed
RUN find . -path "*/src/*.rs" -exec touch {} \;
RUN cargo build --release -p cli

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Non-root user
RUN useradd -ms /bin/bash forgefabrik
USER forgefabrik
WORKDIR /home/forgefabrik

COPY --from=builder /app/target/release/forgefabrik /usr/local/bin/forgefabrik

EXPOSE 8080

ENTRYPOINT ["forgefabrik"]
CMD ["serve", "--port", "8080", "--seed", "42"]
