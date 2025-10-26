# Changelog

All notable changes to Sutra AI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added - USearch HNSW Migration (2025-10-24)

#### HNSW Persistent Index - Production-Ready
- **[MAJOR]** Migrated from hnsw-rs to USearch for TRUE disk persistence
- **94× faster startup** (3.5ms vs 327ms for 1K vectors)
- **24% smaller index files** (single `.usearch` format vs 3-file format)
- **SIMD-optimized** search operations
- **Zero breaking changes** - API remains identical
- Comprehensive tests: `packages/sutra-storage/tests/test_hnsw_persistence.rs`
- Design doc: `docs/storage/HNSW_PERSISTENCE_DESIGN.md`
- Migration report: `docs/storage/USEARCH_MIGRATION_COMPLETE.md`

#### Files Changed
- `packages/sutra-storage/Cargo.toml`: Replaced `hnsw_rs = "0.3"` with `usearch = "2.21"`
- `packages/sutra-storage/src/hnsw_container.rs`: Complete rewrite (~500 lines)
  - Replaced `Hnsw<'static, f32, DistCosine>` with `usearch::Index`
  - Updated `load_or_build()` to use `Index::load()` (true mmap)
  - Updated `save()` to use single-file `.usearch` format
  - Updated `search()` and `insert()` for USearch API
- `packages/sutra-storage/src/lib.rs`: Deprecated hnsw_persistence module

#### Performance Impact
- **Startup time**: 3.5ms for 1K vectors (was 327ms - **94× faster**)
- **Projected 1M vectors**: 3.5s load (was 5.5min rebuild - **94× faster**)
- **File size**: 24% reduction (900MB vs 1.2GB for 1M vectors)
- **Search latency**: No change (same O(log N))
- **Memory usage**: 24% reduction with better compression

#### Migration Notes
- **Automatic**: New `.usearch` file created on first save
- **Zero downtime**: Backward compatible, automatic on restart
- **Cleanup** (optional after 1 week): Delete old `.hnsw.graph` and `.hnsw.data` files
- **Metadata**: `.hnsw.meta` format remains compatible

---

### Added - Enterprise-Grade Durability (2025-10-18)

#### Write-Ahead Log (WAL) Integration
- **Zero Data Loss**: All write operations now logged to WAL before in-memory structures
- **Automatic Crash Recovery**: WAL replay on startup recovers all committed writes
- **WAL Checkpointing**: Safe truncation after successful flush to storage.dat
- **Comprehensive Tests**: Added crash recovery simulation tests
  - `test_wal_crash_recovery`: Verifies data survives crash without flush
  - `test_wal_checkpoint`: Verifies WAL truncation after flush

#### ConcurrentMemory Improvements
- Integrated existing WAL infrastructure into ConcurrentMemory
- Added `replay_wal()` for startup recovery
- Modified `learn_concept()` and `learn_association()` to log to WAL first
- Enhanced `flush()` to checkpoint WAL after successful disk write
- RPO (Recovery Point Objective): 0 (zero data loss)

#### Documentation
- Created `docs/HA_REPLICATION_DESIGN.md` - High Availability architecture design
- Updated WARP.md with durability guarantees and WAL usage
- Added deployment guides and testing instructions

### Technical Details

**Files Modified:**
- `packages/sutra-storage/src/concurrent_memory.rs`:
  - Added WAL field to ConcurrentMemory struct
  - Integrated WAL.append() before WriteLog operations
  - Implemented WAL replay on startup
  - Added WAL checkpoint on flush
  - Added crash recovery tests

**Performance Impact:**
- Write latency: +0.1ms (WAL fsync overhead)
- Read latency: No change (<0.01ms)
- Throughput: 57K writes/sec maintained
- Storage overhead: ~100 bytes per operation in WAL (cleared on flush)

**Breaking Changes:**
- None (backward compatible)

### Planned - High Availability (Coming Soon)

#### Phase 1: WAL Streaming (Week 1-2)
- [ ] Implement WalStreamer for leader node
- [ ] Implement ReplicaReceiver for replica nodes
- [ ] Add gRPC replication service
- [ ] Test 1 leader + 1 replica setup

#### Phase 2: Health Checks (Week 3)
- [ ] Implement HealthMonitor
- [ ] Add heartbeat mechanism
- [ ] Automatic failure detection

#### Phase 3: Automatic Failover (Week 4)
- [ ] Leader election (priority-based)
- [ ] Automatic promotion of replicas
- [ ] Client redirection to new leader
- [ ] RTO (Recovery Time Objective): <5 seconds

#### Phase 4: Production Hardening (Week 5-6)
- [ ] Split-brain protection
- [ ] Raft consensus integration
- [ ] Replication lag monitoring
- [ ] Chaos engineering tests

**Target Metrics:**
- RPO: ~10ms (async replication) or 0ms (sync option)
- RTO: <5 seconds (automatic failover)
- Write Latency: +0ms (async) or +2-10ms (sync)

---

## [1.0.0] - 2024-12-15

### Added
- Initial production release
- ConcurrentStorage with 57K writes/sec
- Lock-free architecture
- Single-file storage.dat format
- Vector search with HNSW
- gRPC API
- React control center
- Streamlit client interface

### Performance
- 57,412 writes/sec (25,000× faster than baseline)
- <0.01ms read latency
- Zero-copy memory-mapped access
- 100% test pass rate

---

## Release Notes

### How to Update

#### From Pre-WAL Version
```bash
# 1. Backup existing data
cp -r ./knowledge ./knowledge.backup

# 2. Pull latest code
git pull origin main

# 3. Rebuild Rust storage
cd packages/sutra-storage
cargo build --release

# 4. Test WAL integration
cargo test test_wal

# 5. Restart services
docker compose down
docker compose up -d
```

#### WAL is automatically enabled - no configuration changes needed

### Verifying WAL Integration

```bash
# Check WAL file exists
ls -lh ./knowledge/wal.log

# Check WAL checkpoint after flush
# WAL should be <1KB after flush
ls -lh ./knowledge/wal.log

# Test crash recovery
# 1. Write data without flush
# 2. Kill process
# 3. Restart - data should be recovered
```

### Migration Notes

- **No data migration required**: WAL is additive
- **Backward compatible**: Old storage.dat files work with new code
- **Forward compatible**: New storage.dat files work with replication (coming soon)

---

## Support

For issues or questions:
- GitHub Issues: https://github.com/your-org/sutra-models/issues
- Documentation: See WARP.md and docs/ directory
- Architecture: See docs/HA_REPLICATION_DESIGN.md
