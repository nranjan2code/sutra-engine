# Sutra AI - Quickstart Guide

**Get started with Sutra AI's conversation-first interface in 5 minutes**

---

## 🚀 Quick Start

### Prerequisites

- Docker and Docker Compose installed
- 8GB+ RAM available
- Ports 8080, 8000, 50051-50053 available

### 1. Deploy Sutra AI

```bash
# Clone the repository
git clone https://github.com/yourusername/sutra-models.git
cd sutra-models

# Deploy all services (one command)
sutra deploy

# Check status
sutra status
```

Expected output:
```
✅ sutra-api: healthy (port 8000)
✅ sutra-client: healthy (port 8080)
✅ sutra-storage: healthy (port 50051)
✅ sutra-user-storage: healthy (port 50053)
✅ sutra-embedding-service: healthy (port 8888)
```

### 2. Access the Web Interface

Open your browser: **http://localhost:8080**

### 3. Create Your Account

1. Click **"Sign Up"** on the login page
2. Enter your email and choose a strong password
3. Create or join an organization

### 4. Start Your First Conversation

1. You'll see the chat interface
2. Type a question in the message box
3. Press **Enter** or click **Send**
4. Watch as Sutra AI:
   - Loads context from your knowledge
   - Queries relevant information
   - Reasons through the answer
   - Streams the response in real-time

### 5. Explore Key Features

**Keyboard Shortcuts:**
- Press **`Cmd+K`** (Mac) or **`Ctrl+K`** (Windows/Linux): Open command palette
- Press **`?`**: View all keyboard shortcuts

**Create Spaces:**
- Click the **"+"** button in the sidebar
- Organize conversations by topic or project

**View Reasoning:**
- Click **"Show reasoning"** on any AI response
- See the knowledge graph and reasoning steps

---

## 📚 Next Steps

- **[User Guide](./USER_GUIDE.md)**: Complete feature walkthrough
- **[API Reference](./API_REFERENCE.md)**: REST API documentation
- **[FAQ](./FAQ.md)**: Common questions and troubleshooting

---

## 🛠️ Common Commands

```bash
# View logs
sutra logs sutra-api
sutra logs sutra-client

# Restart a service
sutra restart sutra-api

# Stop all services
sutra stop

# Update to latest version
docker-compose -f .sutra/compose/production.yml up -d --build

# Check health
curl http://localhost:8000/health
```

---

## 🔒 Security Notes

**Development Mode (Default):**
- No TLS encryption
- For localhost use only
- Not suitable for production

**Production Mode:**
- Set `SUTRA_SECURE_MODE=true`
- Configure JWT secret: `SUTRA_JWT_SECRET_KEY`
- Enable HTTPS with reverse proxy (nginx, Caddy, Traefik)
- See [Production Deployment Guide](./PRODUCTION_DEPLOYMENT.md)

---

## 💡 First Use Tips

1. **Import Your Data:**
   - Use the bulk ingester to load existing documents
   - See `packages/sutra-bulk-ingester/README.md`

2. **Organize with Spaces:**
   - Create spaces for different topics (e.g., "Legal", "Medical", "Engineering")
   - Each space can have its own domain storage

3. **Collaborate:**
   - Invite team members to your organization
   - Assign roles: Admin, Writer, or Reader

4. **Search Everything:**
   - Use `Cmd+K` to search across all conversations, messages, and spaces
   - Semantic search understands meaning, not just keywords

5. **Trust the Reasoning:**
   - Every answer shows its sources
   - View the knowledge graph to see how conclusions were reached
   - Confidence scores indicate certainty

---

## 📞 Support

- **Documentation**: `docs/` directory
- **Issues**: GitHub Issues
- **Community**: Discord (coming soon)

---

## 🎯 What's Next?

You're ready to use Sutra AI! Here's what to explore:

1. ✅ **Create your first conversation** (you just did this!)
2. 📁 **Create a space** to organize your work
3. 🔍 **Try the command palette** with `Cmd+K`
4. 🧠 **View reasoning graphs** to understand how answers are derived
5. 📤 **Import your knowledge** using the bulk ingester

**Happy reasoning!** 🚀
