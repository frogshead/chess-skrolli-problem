# Knight Random Walk — Skrolli Problem

Monte Carlo simulation for a Finnish math competition problem (Skrolli magazine).

> A knight starts at the bottom-left corner of a board. Each turn it picks a random legal move
> with equal probability. The game ends when it reaches the top-right corner (win) or
> bottom-right corner (lose). What is the win probability for a **3×3** and **100×100** board?
> Give the answer to 10 decimal places.

## Browser demo

Open `www/index.html` via a local HTTP server (WASM requires HTTP, not `file://`):

```bash
wasm-pack build --target web --out-dir www/pkg
python3 -m http.server 8080 --directory www/
# open http://localhost:8080
```

- **Animate** — watch a single knight game step-by-step
- **Heatmap** — run N iterations and see color-coded visit frequencies
- **Speed** slider controls animation delay

## CLI

```bash
cargo run                                    # default: 100k iterations, 8×8 board
cargo run -- --board-size 3                  # 3×3 board
cargo run -- --heatmap --board-size 8        # visit-frequency heatmap in terminal
cargo run -- --visualize --board-size 5      # step-by-step animation in terminal
cargo run -- --iterations 1000000           # more iterations for higher precision
```

## Development

```bash
cargo build       # build
cargo test        # run tests
cargo clippy      # lint
```

## Docker

Build and run locally:

```bash
docker build -t chess-skrolli-problem .
docker run -p 8080:80 chess-skrolli-problem
# open http://localhost:8080
```

Or use the pre-built image from GHCR:

```bash
docker compose up -d   # pulls ghcr.io/frogshead/chess-skrolli-problem:latest
```

The image is built and pushed automatically to GHCR on every push to `master`.

## Architecture

All code lives in two files:

| File | Purpose |
|------|---------|
| `src/lib.rs` | Core simulation logic + WebAssembly bindings (`WasmGame`) |
| `src/main.rs` | CLI, terminal visualization (crossterm), argument parsing (clap) |

The board uses (0,0)-based coordinates where `x` increases right and `y` increases up.
Start = `(0,0)`, win = `(size-1, size-1)`, lose = `(size-1, 0)`.
