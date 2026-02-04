# Standalone Quickstart

This is the minimal path to run Sutra Engine as a standalone storage service.

---

## 1. Build

```bash
cargo build --release --bin storage-server
```

## 2. Run

```bash
./target/release/storage-server
```

Defaults:
- TCP listen: `0.0.0.0:50051`
- Storage path: `/data/storage.dat`

Override with env vars:

```bash
export STORAGE_PATH=./data
export STORAGE_PORT=50051
export VECTOR_DIMENSION=768
export SUTRA_STORAGE_MODE=single
```

## 3. Minimal Client Check

The protocol crate includes a tiny roundtrip example:

```bash
cargo run -p sutra-protocol --example minimal_roundtrip
```

## 4. Docker (Optional)

```bash
docker compose -f infra/docker/standalone.yml up --build
```

---

## Next Steps

- Full API details: `docs/API_REFERENCE.md`
- Operational guidance: `docs/OPERATIONS.md`
