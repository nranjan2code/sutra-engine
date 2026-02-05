# Security Manual: Hardening Sutra Engine

Sutra Engine is designed to be secure by default when configured for production. 

---

## 🔒 Security Architecture

The engine implements three layers of security:
1. **Transport Layer Security (TLS 1.3)**: Encrypts all traffic between client and server.
2. **Authentication (HMAC-SHA256)**: Verifies the identity of the client.
3. **Authorization (Claims)**: Restricts clients to specific operations (Read/Write/Admin).

---

## 🔑 Authentication (HMAC)

When `SUTRA_SECURE_MODE=true` is set, every TCP request must be signed.

### Server Configuration
```bash
export SUTRA_SECURE_MODE=true
export SUTRA_AUTH_SECRET="your-32-character-minimal-secure-key"
./start-engine.sh
```

### Protocol Signature
The client must send an authentication header before any command.
- **Algorithm**: HMAC-SHA256
- **Content**: `TIMESTAMP + REQUEST_BODY`
- **Verification**: The server rejects any request with a timestamp older than 300 seconds (preventing replay attacks).

---

## 🛰 Transport Layer Security (TLS)

Sutra uses native Rustls for high-performance encryption.

### Providing Certificates
Generate or provide your `.pem` and `.key` files:
```bash
export SUTRA_TLS_CERT="./certs/server.crt"
export SUTRA_TLS_KEY="./certs/server.key"
./start-engine.sh
```

---

## 🚦 Rate Limiting

To prevent DoS attacks, you can limit the number of requests per second.

```bash
export SUTRA_RATE_LIMIT_RPS=1000
export SUTRA_RATE_LIMIT_BURST=5000
```

If a client exceeds these limits, the engine will return a specialized `Error` response and throttle the connection.

---

## 🤖 Autonomy Request Authorization

When running in secure mode, autonomy-related requests are categorized as follows:

| Category | Requests |
|----------|----------|
| **Read** | `ListSubscriptions`, `ListGoals`, `GetAutonomyStats` |
| **Write** | `Subscribe`, `Unsubscribe`, `CreateGoal`, `ProvideFeedback` |
| **Delete** | `CancelGoal` |

Clients must have the appropriate claim level to perform these operations.

---

## 🛡 Network Hardening

1. **Private Network**: Always run Sutra in a private subnet.
2. **Firewall**: Lock down port `50051` to known client IP addresses.
3. **Dedicated User**: Run the binary as a non-root user (e.g., `sutra`).
   ```bash
   useradd -m sutra
   sudo -u sutra ./start-engine.sh
   ```

---
