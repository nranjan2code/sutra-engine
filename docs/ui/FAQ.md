# Sutra AI - Frequently Asked Questions (FAQ)

**Quick answers to common questions about Sutra AI**

---

## General

### What is Sutra AI?

Sutra AI is an **explainable AI system** designed for regulated industries (medical, legal, financial). Unlike general chatbots, it:

- Starts empty and learns from YOUR proprietary data
- Shows complete reasoning paths for every answer
- Provides confidence scores and "I don't know" responses
- Stores everything in knowledge graphs with full audit trails

### How is Sutra AI different from ChatGPT/Claude?

| Feature | Sutra AI | ChatGPT/Claude |
|---------|----------|----------------|
| **Knowledge Source** | Your proprietary data | General internet training |
| **Explainability** | Full reasoning graphs | Black box |
| **Confidence** | Scored (0-100%) | No scores |
| **"I don't know"** | Yes, when uncertain | Often guesses |
| **Audit Trail** | Complete | None |
| **Compliance** | FDA/HIPAA/SOC2 ready | Not designed for compliance |
| **Domain-Specific** | Learns your domain | General knowledge |

### What industries use Sutra AI?

- **Healthcare**: Medical protocols, diagnosis support, drug interactions
- **Legal**: Case law research, contract analysis, regulatory compliance
- **Financial**: Risk assessment, compliance monitoring, fraud detection
- **Engineering**: Technical documentation, troubleshooting, design review
- **Pharmaceuticals**: Clinical trial data, regulatory submissions

### Is my data private?

**Yes, completely:**
- All data stored locally in your infrastructure
- No data sent to third parties
- You control the servers
- End-to-end encryption available (production mode)

---

## Getting Started

### What are the system requirements?

**Server:**
- Docker and Docker Compose
- 8GB+ RAM (16GB recommended)
- 20GB+ disk space
- Linux, macOS, or Windows with WSL2

**Client:**
- Modern web browser (Chrome 90+, Firefox 88+, Safari 14+)
- JavaScript enabled
- Internet connection (to access your server)

### How do I install Sutra AI?

```bash
# One command deployment
./sutra-deploy.sh install

# Check status
./sutra-deploy.sh status

# Access UI
open http://localhost:8080
```

See [QUICKSTART.md](./QUICKSTART.md) for full details.

### Can I run this on Windows?

Yes, with **WSL2 (Windows Subsystem for Linux)**:
1. Install WSL2
2. Install Docker Desktop for Windows
3. Run deployment commands in WSL2 terminal

### Do I need a GPU?

**No**, Sutra AI runs on CPU:
- Embedding generation works on CPU
- Optimized for production inference
- GPU optional for faster large-scale imports

---

## Authentication

### How do I create an account?

1. Navigate to `http://localhost:8080`
2. Click **"Sign Up"**
3. Enter email, password, and organization name
4. Click **"Create Account"**

### What are the password requirements?

- Minimum 8 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one number
- Special characters recommended

### How long does my session last?

- **Default**: 24 hours
- Automatically refreshes while active
- Can be configured via `SUTRA_JWT_EXPIRATION_HOURS`

### Can I reset my password?

**Currently**: Password reset not implemented (coming soon)

**Workaround**: Admin can manually update password in user-storage.dat

### Can multiple people use the same account?

**Not recommended**:
- Each person should have their own account
- Enables proper audit trails
- Tracks individual actions
- Better security

---

## Conversations

### How do I start a new conversation?

**Method 1**: Click **"+ New Chat"** in sidebar  
**Method 2**: Press **`Cmd+N`** (Mac) or **`Ctrl+N`** (Windows/Linux)

### Why doesn't the AI answer my question?

**Possible reasons:**

1. **Confidence too low**: AI says "I don't know" instead of guessing
   - **Solution**: Provide more context or rephrase

2. **No relevant knowledge**: Knowledge base doesn't contain the information
   - **Solution**: Import relevant documents

3. **Out of scope**: Question outside the domain
   - **Solution**: Check you're in the correct space

4. **Ambiguous question**: Question too vague
   - **Solution**: Be more specific

### What does the confidence score mean?

- **🟢 High (>80%)**: Very confident, strong evidence
- **🟡 Medium (50-80%)**: Moderate confidence, some uncertainty
- **🔴 Low (<50%)**: Uncertain, may respond "I don't know"

**Note**: Low confidence is a feature, not a bug. It prevents hallucinations.

### Can I edit messages after sending?

**Not currently**:
- Messages are immutable (for audit trails)
- Start a new message to clarify or correct

**Workaround**: Create a new conversation for clean slate

### How do I delete a conversation?

1. Hover over conversation in sidebar
2. Click **trash icon**
3. Confirm deletion

**Note**: Deletion is soft (recoverable by admins for audit compliance)

### Where are conversations stored?

- **User conversations**: `data/user-storage/storage.dat`
- **Domain knowledge**: `data/storage/storage.dat` (separate)
- **Encrypted**: In production mode with `SUTRA_SECURE_MODE=true`

---

## Spaces

### What is a space?

A space is an organizational unit for grouping conversations:
- Like folders for conversations
- Can have different team members
- Can use different knowledge bases
- Helps organize by project/topic/client

### Do I need to create spaces?

**No**, you get a **Personal Space** automatically:
- Private workspace just for you
- Good for personal projects
- Can create additional spaces anytime

### How many spaces can I create?

**Unlimited** (within system resources)

### Who can see my conversations in a space?

**Depends on space type:**

- **Personal Space**: Only you
- **Shared Space**: All members with appropriate roles
  - **Admin**: Full access
  - **Writer**: Read/write
  - **Reader**: Read-only

### Can I move conversations between spaces?

**Not currently** (coming soon)

**Workaround**: Copy important messages to new conversation in target space

### What happens if I delete a space?

- **Soft delete**: Space marked as deleted but data preserved
- **Conversations**: Remain in database (for audit)
- **Access**: No longer visible to users
- **Recovery**: Admin can restore if needed

---

## Search

### How does search work?

**Semantic search** using the storage engine:
- Understands meaning, not just keywords
- Example: "legal precedent" finds "case law"
- Searches conversations, messages, and spaces

### How do I search?

**Keyboard**: **`Cmd+K`** (Mac) or **`Ctrl+K`** (Windows/Linux)  
**Mouse**: Click search icon in top bar

### Why doesn't search find what I'm looking for?

**Possible reasons:**

1. **Access denied**: You don't have read access to that space
2. **Not indexed yet**: Very recent content (wait 30 seconds)
3. **Typo**: Check spelling
4. **Too vague**: Be more specific

**Tips for better search:**
- Use natural language: "How do I approve a 510(k)?"
- Be specific: "FDA approval" not just "approval"
- Try synonyms: "precedent" or "case law"

### Can I search only conversations or only messages?

**Yes**, in the command palette:
- Search shows all results by default
- Results grouped by type (Conversations, Messages, Spaces)
- API supports type-specific search (for custom integrations)

### How fast is search?

- **P99 latency**: <60ms (including semantic search)
- **Debounce**: 200-300ms (prevents API spam)
- **Perceived latency**: <500ms total

---

## Knowledge Graphs

### What is a reasoning graph?

A visual representation showing:
- What concepts were used
- How they're related
- Steps in the reasoning process
- Why the answer was reached

### How do I view the reasoning graph?

1. Look for **"Show reasoning"** below any AI message
2. Click to expand
3. View:
   - Reasoning steps (numbered list)
   - Knowledge graph (interactive visualization)
   - Confidence score

### What do the colors mean?

**Node colors:**
- 🟢 **Green**: High confidence (>80%)
- 🟡 **Orange**: Medium confidence (50-80%)
- 🔴 **Red**: Low confidence (<50%)

**Edge styles:**
- **Solid arrows**: Direct relationships
- **Animated**: Active reasoning path
- **Dashed**: Weak or inferred connections

### Why is the graph useful?

1. **Transparency**: See exactly how answer was reached
2. **Trust**: Verify reasoning logic
3. **Compliance**: Required for regulated industries
4. **Learning**: Understand domain relationships
5. **Debugging**: Find gaps in knowledge

### Can I export the graph?

**Currently**: Screenshot only (browser)

**Coming soon**: PNG/SVG export, PDF reports

### The graph is too large and slow. What do I do?

**Solutions:**
1. **Zoom controls**: Use minimap to navigate
2. **Node limit**: We limit to 50 nodes (most relevant)
3. **Collapse branches**: Click nodes to collapse subtrees
4. **Better hardware**: More RAM helps with large graphs

---

## Performance

### Why is the UI slow?

**Possible causes:**

1. **Slow network**: Check connection to server
2. **Large conversation**: 100+ messages
3. **Complex graph**: 50+ nodes
4. **Browser**: Try Chrome/Edge (best performance)
5. **Server resources**: Check Docker memory limits

**Solutions:**
- Use Chrome or Edge
- Close other tabs
- Increase Docker memory: `docker-compose.yml` → `mem_limit: 4g`
- Enable production mode optimizations

### How can I make search faster?

Search is already optimized (<60ms p99), but:
- Use specific queries (fewer results to rank)
- Limit scope to specific space
- Upgrade server hardware (SSD, more RAM)

### Does Sutra AI work offline?

**No**, requires connection to server:
- Server runs AI models
- Knowledge stored on server
- Can deploy server locally on laptop for "offline" use

### What's the maximum conversation length?

**No hard limit**, but:
- Context window: Last 10 messages by default
- Performance: 100+ messages may slow rendering
- Best practice: Start new conversation for new topics

---

## Administration

### How do I add users?

**Method 1 (Self-Registration):**
1. Share instance URL
2. Users sign up themselves
3. Join organization via invite code

**Method 2 (Admin Invite):**
1. Add to space with email
2. User receives invite link
3. Creates account to accept

### How do I manage permissions?

**Space-level permissions:**
1. Open space → **"Manage Space"**
2. View members list
3. Change roles:
   - **Admin**: Full control
   - **Writer**: Read/write
   - **Reader**: Read-only

### Can I backup my data?

**Yes:**

```bash
# Stop services
./sutra-deploy.sh stop

# Backup data volumes
docker run --rm \
  -v user-storage-data:/data \
  -v $(pwd)/backup:/backup \
  alpine tar czf /backup/user-storage-$(date +%Y%m%d).tar.gz /data

# Restart
./sutra-deploy.sh start
```

See [Production Deployment Guide](./PRODUCTION_DEPLOYMENT.md) for full backup procedures.

### How do I restore from backup?

```bash
# Stop services
./sutra-deploy.sh stop

# Restore data
docker run --rm \
  -v user-storage-data:/data \
  -v $(pwd)/backup:/backup \
  alpine sh -c "cd / && tar xzf /backup/user-storage-20241026.tar.gz"

# Restart
./sutra-deploy.sh start
```

### How do I upgrade to a new version?

```bash
# Check current version
./sutra-deploy.sh version

# Pull latest
git pull origin main

# Update containers
./sutra-deploy.sh update

# Verify
./sutra-deploy.sh status
```

### How do I monitor the system?

**Built-in health checks:**
```bash
# Overall status
./sutra-deploy.sh status

# Service logs
./sutra-deploy.sh logs sutra-api
./sutra-deploy.sh logs sutra-storage

# Health endpoints
curl http://localhost:8000/health
curl http://localhost:8000/auth/health
```

**External monitoring** (optional):
- Prometheus metrics (coming soon)
- Grafana dashboards (coming soon)
- Sentry error tracking (configure in production)

---

## Security

### Is Sutra AI secure?

**Development mode (default):**
- ⚠️ **Not secure**: No encryption, no auth on storage
- ✅ **For**: localhost development only

**Production mode:**
- ✅ **Secure**: TLS 1.3, HMAC-SHA256, RBAC
- ✅ **For**: real deployments

**Enable production mode:**
```bash
export SUTRA_SECURE_MODE=true
export SUTRA_JWT_SECRET_KEY=<your-secret-key>
# See PRODUCTION_DEPLOYMENT.md
```

### Should I use HTTPS?

**Yes, in production:**
- Use reverse proxy (nginx, Caddy, Traefik)
- Get SSL certificate (Let's Encrypt is free)
- Redirect HTTP → HTTPS

See [Production Deployment Guide](./PRODUCTION_DEPLOYMENT.md)

### How are passwords stored?

- **Algorithm**: Argon2id (industry best practice)
- **Salt**: Unique per password
- **Never**: Stored in plain text
- **Never**: Returned in API responses

### What about data privacy regulations?

Sutra AI is designed for compliance:

- **HIPAA**: Full audit trails, encryption at rest/transit
- **GDPR**: Data residency (you control servers), right to erasure
- **SOC2**: Audit logs, access controls, encryption
- **FDA**: Complete reasoning transparency, traceability

**Note**: You're responsible for configuring production security settings

### Can I integrate with SSO/SAML?

**Not yet** (coming soon)

**Roadmap:**
- OAuth 2.0 integration
- SAML 2.0 support
- Active Directory integration

---

## Troubleshooting

### "Connection refused" error

**Causes:**
1. Services not running
2. Wrong port
3. Firewall blocking

**Solutions:**
```bash
# Check status
./sutra-deploy.sh status

# Restart services
./sutra-deploy.sh restart

# Check ports
netstat -an | grep 8080
netstat -an | grep 8000
```

### "401 Unauthorized" errors

**Causes:**
1. Session expired
2. Invalid token
3. JWT secret changed

**Solutions:**
1. Log out and log back in
2. Logout to clear httpOnly cookies (server-managed)
3. Clear browser cache if needed (cookies auto-cleared on logout)
3. Check `SUTRA_JWT_SECRET_KEY` consistency

### Messages not appearing

**Causes:**
1. Space filter applied
2. Search filter active
3. Permission issue

**Solutions:**
1. Check space selector (top left)
2. Clear search (if command palette open)
3. Verify you have read access to space

### Docker containers not starting

**Check logs:**
```bash
./sutra-deploy.sh logs sutra-api
./sutra-deploy.sh logs sutra-storage
```

**Common issues:**
- Port conflict: Change ports in `docker-compose-grid.yml`
- Memory limit: Increase Docker memory allocation
- Dependency: Check health checks pass

### UI not loading

**Quick fixes:**
1. Hard refresh: `Cmd+Shift+R` (Mac) or `Ctrl+Shift+R` (Windows)
2. Clear browser cache
3. Try incognito/private window
4. Check browser console for errors (F12)

**Still broken:**
```bash
# Rebuild UI
./sutra-deploy.sh update sutra-client
```

---

## Data Management

### How do I import existing documents?

Use the **bulk ingester**:

```bash
cd packages/sutra-bulk-ingester

# Prepare documents (one per line)
cat your-documents.txt | cargo run -- \
  --storage-url localhost:50051 \
  --batch-size 100

# See progress
tail -f logs/ingestion.log
```

### What file formats are supported?

**Currently**: Plain text (.txt)

**Coming soon:**
- PDF
- Word documents (.docx)
- Markdown (.md)
- JSON structured data

### How do I export my data?

**Conversations:**
- Use API: `GET /conversations/{id}/messages`
- Export to JSON
- See [API_REFERENCE.md](./API_REFERENCE.md)

**Knowledge:**
- Backup `storage.dat` files
- Export via bulk exporter (coming soon)

### How much data can Sutra AI handle?

**Tested scale:**
- 10M+ concepts per storage
- 1M+ conversations
- 100M+ messages
- 16 storage shards (distributed)

**Performance maintained:**
- <0.01ms concept lookup
- <60ms semantic search
- <100ms graph query

---

## Development

### Can I customize Sutra AI?

**Yes**, it's open source:
- Modify UI components (React + TypeScript)
- Add API endpoints (Python FastAPI)
- Extend storage engine (Rust)
- See `CONTRIBUTING.md`

### How do I contribute?

1. Fork the repository
2. Create a feature branch
3. Make changes
4. Write tests
5. Submit pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md)

### Is there an API?

**Yes**, full REST API:
- OpenAPI documentation: `http://localhost:8000/docs`
- See [API_REFERENCE.md](./API_REFERENCE.md)
- All UI features available via API

### Can I build custom integrations?

**Absolutely:**
- REST API for all operations
- WebSocket for streaming (coming soon)
- Webhook support (coming soon)
- Client SDKs (Python, JavaScript - coming soon)

---

## Support

### Where can I get help?

1. **Documentation**: `docs/` directory
2. **FAQ**: This document
3. **Troubleshooting**: `docs/TROUBLESHOOTING.md`
4. **GitHub Issues**: Report bugs
5. **Community**: Discord (coming soon)

### How do I report a bug?

1. Go to GitHub Issues
2. Click **"New Issue"**
3. Select **"Bug Report"**
4. Fill in template:
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Screenshots/logs
   - System info

### How do I request a feature?

1. Go to GitHub Issues
2. Click **"New Issue"**
3. Select **"Feature Request"**
4. Describe:
   - Use case
   - Proposed solution
   - Alternatives considered

### Is there commercial support?

**Coming soon:**
- Enterprise support plans
- Custom development
- Training and onboarding
- SLA guarantees

---

## Roadmap

### What's coming next?

**Q1 2025:**
- [ ] Password reset flow
- [ ] SSO/SAML integration
- [ ] Mobile app (iOS/Android)
- [ ] Improved graph export (PNG/SVG/PDF)

**Q2 2025:**
- [ ] Real-time collaboration
- [ ] Comments and annotations
- [ ] Advanced admin dashboard
- [ ] Prometheus metrics

**Q3 2025:**
- [ ] Multi-modal support (images, audio)
- [ ] Slack/Teams integration
- [ ] Custom workflows
- [ ] Advanced analytics

See `ROADMAP.md` for full details.

---

**Last Updated**: October 26, 2025  
**Version**: 2.0.0

**Have a question not answered here?**  
[Open an issue on GitHub](https://github.com/yourusername/sutra-models/issues)
