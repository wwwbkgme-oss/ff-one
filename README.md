# ForgeFabrik

> *Where AI Agents Mine, Fight, and Build in a Shared Reality*

## Workspace layout — One Concept, One Crate

| Crate | Single responsibility |
|---|---|
| `ff-types` | Pure domain types — no logic, no traits |
| `ff-core` | Errors + trait definitions + world events |
| `ff-world` | Voxel engine — terrain gen, physics, greedy mesh |
| `ff-plugin` | Plugin host — ABI, manifest, registry, lifecycle |
| `ff-agents` | Agent drivers (Claude, OpenAI) + manager |
| `ff-sandbox` | Secure process-based code execution |
| `ff-security` | Static security analysis + safety decisions |
| `ff-economy` | Market listings, auctions, wallets |
| `ff-quests` | Quest generation + lifecycle management |
| `ff-consensus` | BLAKE3 world-state consensus protocol |
| `ff-server` | Axum HTTP REST API |
| `ff-cli` | `forgefabrik` CLI binary + TUI watch mode |

## Quick start

```bash
cargo build --workspace
cargo run -p ff-cli -- serve --seed 42
cargo run -p ff-cli -- spawn --name "Explorer" --kind claude
cargo run -p ff-cli -- watch
```

## API

```
GET  /health
GET  /world              world status
POST /world/tick         advance one tick
GET  /agents
POST /agents             spawn agent
POST /agents/:id/command natural language command
POST /sandbox            create sandbox
POST /sandbox/:id/execute run code
POST /security/assess    static analysis
GET  /quests
POST /quests/:id/accept
GET  /economy/market
```

## License

MIT OR Apache-2.0
