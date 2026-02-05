# Troubleshooting Guide

Common issues, error codes, and solutions for Sutra Engine.

---

## 🚀 Startup Issues

### `Failed to create WAL: No such file or directory`
**Cause**: The `STORAGE_PATH` (default `./data`) directory does not exist or is not writable.
**Fix**: 
```bash
mkdir -p ./data
chmod +w ./data
```

### `Address already in use (os error 48)`
**Cause**: Another process is already running on port `50051`.
**Fix**: 
- Kill the existing process: `pkill sutra-engine`
- Or change the port: `export STORAGE_PORT=50052`

---

## 🛰 Connection Issues

### `Connection timeout after 30 seconds`
**Cause**: "Connection Storm" – too many clients creating new TCP connections simultaneously.
**Fix**: Use **Connection Pooling** in your client. (The Sutra Python SDK does this automatically).

### `Unauthorized: Insufficient permissions`
**Cause**: You are running in Secure Mode, but your client is not sending the correct HMAC signature.
**Fix**: Ensure `SUTRA_AUTH_SECRET` matches on both client and server.

---

## 🧠 Functional Issues

### `Search returns 0 results`
**Cause**:
1. No concepts match the query.
2. The concepts were added with `generate_embedding: false` and no manual vector was provided.
3. The embedding service (e.g., Hugging Face) is down.
**Fix**: Check engine logs for `Batch embedding failed`.

### `Slow response times (>50ms)`
**Cause**: High CPU load or massive reconciler backlog.
**Fix**:
- Check system CPU/ID usage.
- Increase `MEMORY_THRESHOLD` to reduce disk flushing frequency.
- Ensure you are using the `release` binary, not the `debug` one.

---

## 🤖 Autonomy Issues

### `Autonomy engine disabled` in logs
**Cause**: `SUTRA_AUTONOMY` env var is set to `false`.
**Fix**: Set `SUTRA_AUTONOMY=true` or remove the variable (defaults to enabled).

### High CPU usage from background loops
**Cause**: Autonomy background threads (decay, reasoning, gap detection) are running at default intervals on a large graph.
**Fix**:
- Disable features you don't need: set `SUTRA_AUTONOMY=false` to disable all, or configure individual features programmatically via `AutonomyConfig`.
- Background reasoning and gap detection are the most CPU-intensive. Their intervals can be increased for large graphs.

### `Goal triggered` log messages
**Cause**: A goal's condition was met and its action was executed. This is normal behavior.
**Info**: Check `echo "list goals" | nc localhost 9000` to see goal statuses.

---

## 📝 Log Analysis

Sutra uses the standard `tracing` crate. To see detailed debug logs:
```bash
export RUST_LOG=debug
./sutra-engine
```

**Key Log Indicators:**
- `🆕 No existing storage found`: First run initialized properly.
- `🔄 Replaying WAL`: Engine recovered from a previous shutdown.
- `🚀 Adaptive reconciler started`: Background tasks are running.
- `Starting autonomy engine...`: Autonomy background loops initializing.
- `Autonomy engine started`: All enabled autonomy features are running.
- `Reasoning cycle: N new associations, N contradictions, N strengthened`: Normal reasoning activity.
- `❌ Serialization error`: Client-server protocol mismatch.

---
