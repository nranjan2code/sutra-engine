# Sutra AI - User Guide

**Complete guide to using Sutra AI's conversation-first interface**

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Authentication](#authentication)
3. [Conversations](#conversations)
4. [Spaces](#spaces)
5. [Search](#search)
6. [Knowledge Graphs](#knowledge-graphs)
7. [Keyboard Shortcuts](#keyboard-shortcuts)
8. [Best Practices](#best-practices)

---

## Getting Started

### System Requirements

- Modern web browser (Chrome 90+, Firefox 88+, Safari 14+, Edge 90+)
- JavaScript enabled
- Stable internet connection (for API requests)

### Accessing Sutra AI

1. Navigate to your Sutra AI instance: `http://localhost:8080` (development) or your production URL
2. Log in or create an account
3. You'll be directed to the main chat interface

---

## Authentication

### Creating an Account

1. Click **"Sign Up"** on the login page
2. Fill in the registration form:
   - **Email**: Your work or personal email
   - **Password**: Minimum 8 characters, must include uppercase, lowercase, and numbers
   - **Organization**: Create new or join existing
3. Click **"Create Account"**
4. You'll be automatically logged in

### Logging In

1. Enter your email and password
2. Click **"Log In"**
3. Your session lasts 24 hours (configurable)

### Password Requirements

- Minimum 8 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one number
- Recommended: special characters for extra security

### Session Management

- Sessions automatically refresh
- Click your avatar (top right) → **"Logout"** to end session
- Multiple device login supported

---

## Conversations

### Starting a New Conversation

**Method 1: New Chat Button**
1. Click **"+ New Chat"** in the sidebar
2. Start typing your first message

**Method 2: Keyboard Shortcut**
1. Press **`Cmd+N`** (Mac) or **`Ctrl+N`** (Windows/Linux)
2. New conversation opens automatically

### Sending Messages

1. Type your message in the input box at the bottom
2. Press **Enter** to send
3. Press **Shift+Enter** to add a new line without sending

### Understanding AI Responses

**Streaming Responses:**
- AI responses appear character by character
- Progress indicators show reasoning stages:
  - 🧠 **Loading Context**: Retrieving relevant information
  - 🔍 **Querying Knowledge**: Searching the knowledge graph
  - 💡 **Reasoning**: Generating the answer

**Confidence Levels:**
- 🟢 **High (>80%)**: Very confident in the answer
- 🟡 **Medium (50-80%)**: Moderate confidence
- 🔴 **Low (<50%)**: Uncertain, may need more information

**"I Don't Know" Responses:**
- Sutra AI won't guess if confidence is too low
- This is a feature, not a bug (prevents hallucinations)
- Try rephrasing or providing more context

### Viewing Conversation History

- All conversations appear in the sidebar
- Grouped by date:
  - **Today**
  - **Yesterday**
  - **Last 7 Days**
  - **Older**

### Organizing Conversations

**Star Important Conversations:**
1. Hover over a conversation in the sidebar
2. Click the **star icon**
3. Starred conversations appear at the top

**Edit Conversation Title:**
1. Click on a conversation
2. Click the **edit icon** next to the title
3. Type a new title
4. Press **Enter** to save

**Delete Conversations:**
1. Hover over a conversation in the sidebar
2. Click the **trash icon**
3. Confirm deletion
4. Note: Deletion is soft (can be recovered by admins)

### Loading More Messages

- Scroll up to see earlier messages
- Infinite scroll automatically loads more
- No "Load More" button needed

---

## Spaces

Spaces help you organize conversations by topic, project, or team.

### What Are Spaces?

- **Personal Space**: Your private workspace (created automatically)
- **Shared Spaces**: Collaborate with team members
- **Domain-Specific**: Each space can use different knowledge storage

### Creating a Space

1. Click the **space selector** dropdown (top left)
2. Click **"Create Space"**
3. Fill in the details:
   - **Name**: Descriptive name (e.g., "Medical Research")
   - **Description**: What this space is for
   - **Icon**: Choose an emoji or icon
   - **Domain Storage**: Select which knowledge base to use
4. Click **"Create"**

### Switching Spaces

1. Click the **space selector** dropdown
2. Select a space from the list
3. Conversations filter to that space automatically

### Space Roles

**Admin:**
- Full control over space
- Can add/remove members
- Can change space settings
- Can delete space

**Writer:**
- Can create and edit conversations
- Can view all space content
- Cannot manage members or settings

**Reader:**
- View-only access
- Can read conversations
- Cannot create or edit

### Managing Space Members

**Adding Members:**
1. Open the space selector
2. Click **"Manage Space"**
3. Click **"Add Member"**
4. Enter email and select role
5. Click **"Invite"**

**Changing Roles:**
1. Open space management
2. Find the member
3. Click their role dropdown
4. Select new role

**Removing Members:**
1. Open space management
2. Find the member
3. Click **"Remove"**
4. Confirm removal
5. Note: Cannot remove the last admin

### Space Settings

Access via space dropdown → **"Manage Space"**:
- **General**: Name, description, icon
- **Domain Storage**: Which knowledge base to use
- **Members**: User access control
- **Danger Zone**: Delete space (admins only)

---

## Search

Sutra AI provides powerful semantic search across all your content.

### Opening Search (Command Palette)

**Keyboard Shortcut:**
- Mac: **`Cmd+K`**
- Windows/Linux: **`Ctrl+K`**

**Or:**
- Click the **search icon** in the top bar

### How Search Works

**Semantic Understanding:**
- Searches by meaning, not just keywords
- Example: "legal precedent" finds "case law" and "judicial ruling"

**What Gets Searched:**
- Conversations (titles and metadata)
- Messages (all content)
- Spaces (names and descriptions)

**Relevance Scoring:**
Search results ranked by:
1. **Semantic similarity** (how closely related)
2. **Recency** (newer results boosted)
3. **Starred status** (starred items appear higher)
4. **Exact matches** (keyword matches highlighted)

### Search Results

**Result Types:**
- 💬 **Conversations**: Shows title, message count, last updated
- 📝 **Messages**: Shows snippet with context
- 📁 **Spaces**: Shows description and conversation count

**Result Actions:**
- **Click** to open
- **Hover** to see full preview
- See **relevance score** badge (0-100)

### Keyboard Navigation

- **↑↓ Arrow Keys**: Navigate results
- **Enter**: Open selected result
- **Esc**: Close search
- **Tab**: Cycle through groups

### Search Tips

1. **Be Specific**: "FDA approval process" > "approval"
2. **Use Natural Language**: "How do I..." works great
3. **Search by Author**: "messages from john@example.com"
4. **Search by Date**: "conversations last week"
5. **Filter by Space**: Select space first, then search

---

## Knowledge Graphs

Every AI response is backed by a reasoning graph showing how the answer was derived.

### Viewing Reasoning

1. Look for **"Show reasoning"** below any AI message
2. Click to expand
3. View:
   - **Reasoning Steps**: Numbered list of logic
   - **Knowledge Graph**: Visual representation
   - **Confidence Score**: How certain the AI is

### Understanding the Graph

**Nodes (Circles):**
- **Green**: High confidence concepts
- **Orange**: Medium confidence
- **Red**: Low confidence or uncertain
- **Size**: Importance in reasoning

**Edges (Arrows):**
- Show relationships between concepts
- Animated: Active reasoning paths
- Static: Supporting context

**Node Types:**
- **Concept**: Domain knowledge
- **Fact**: Verified information
- **Inference**: Derived conclusion
- **Question**: Query context

### Interacting with the Graph

**Pan and Zoom:**
- **Click + Drag**: Pan around
- **Scroll**: Zoom in/out
- **Fit View** button: Center and fit all nodes

**Inspect Nodes:**
- **Click** a node to see full details:
  - Content
  - Confidence score
  - Source (if applicable)
  - Related concepts

**Minimap:**
- Shows overview of entire graph
- Blue box: Current view
- Click to jump to areas

### Why Graphs Matter

**Explainability:**
- See exactly how answers were reached
- Verify reasoning logic
- Build trust in AI outputs

**Compliance:**
- Required for FDA, HIPAA, SOC2
- Complete audit trail
- Traceable decision paths

**Learning:**
- Understand domain relationships
- Discover gaps in knowledge
- Improve knowledge base

---

## Keyboard Shortcuts

Press **`?`** or **`Cmd+/`** to view this list in-app.

### Navigation

| Shortcut | Action |
|----------|--------|
| `Cmd+K` / `Ctrl+K` | Open command palette (search) |
| `Cmd+N` / `Ctrl+N` | New conversation |
| `Cmd+B` / `Ctrl+B` | Toggle sidebar |
| `↑` / `↓` | Navigate conversations/search results |
| `Enter` | Open selected item |
| `Esc` | Close modal/dialog |

### Chat

| Shortcut | Action |
|----------|--------|
| `Enter` | Send message |
| `Shift+Enter` | New line (no send) |
| `Cmd+↑` / `Ctrl+↑` | Scroll to top of conversation |
| `Cmd+↓` / `Ctrl+↓` | Scroll to bottom |

### General

| Shortcut | Action |
|----------|--------|
| `?` | Show keyboard shortcuts |
| `Cmd+/` / `Ctrl+/` | Show help menu |
| `Cmd+,` / `Ctrl+,` | Open settings |

---

## Best Practices

### Getting Better Answers

1. **Be Specific**: "Explain FDA 510(k) clearance" vs "Tell me about FDA"
2. **Provide Context**: "In the context of medical devices..."
3. **Ask Follow-ups**: Build on previous answers
4. **Check Confidence**: Low confidence = rephrase or clarify

### Organizing Your Work

1. **Use Spaces**: Group by project, client, or topic
2. **Star Important**: Mark key conversations for easy access
3. **Descriptive Titles**: Rename conversations for clarity
4. **Archive Old**: Keep active list manageable

### Collaborating with Teams

1. **Shared Spaces**: Create for team projects
2. **Assign Roles**: Give appropriate permissions
3. **Document Decisions**: Use conversations as records
4. **Export Important**: Save critical reasoning graphs

### Maintaining Quality

1. **Review Reasoning**: Check graphs for accuracy
2. **Report Errors**: Flag incorrect answers
3. **Update Knowledge**: Add new information regularly
4. **Monitor Confidence**: Track answer quality over time

### Security & Privacy

1. **Strong Passwords**: Follow password requirements
2. **Log Out**: End sessions on shared computers
3. **Limit Access**: Only invite necessary members
4. **Regular Audits**: Review space membership

---

## Troubleshooting

### Common Issues

**Can't Log In:**
- Verify email/password
- Check caps lock
- Try password reset

**Messages Not Sending:**
- Check internet connection
- Refresh page
- View browser console for errors

**Search Not Working:**
- Check spelling (semantic search helps but...)
- Try different keywords
- Ensure you have read access to spaces

**Graph Not Loading:**
- Large graphs take time
- Try refreshing
- Check browser console

### Getting Help

1. **Documentation**: Check `docs/` directory
2. **FAQ**: See [FAQ.md](./FAQ.md)
3. **Support**: Contact your admin
4. **Logs**: `sutra logs sutra-api`

---

## Advanced Features

### Custom Domain Storage

Each space can use a different knowledge base:
1. Create multiple storage servers
2. Assign in space settings
3. Isolate domain knowledge

### Bulk Data Import

Import existing documents:
```bash
# See bulk ingester documentation
cd packages/sutra-bulk-ingester
cargo run -- --help
```

### API Access

Integrate with external tools:
- REST API: `http://localhost:8000/docs`
- See [API_REFERENCE.md](./API_REFERENCE.md)

### Audit Logs

Track all actions (admin feature):
- User login/logout
- Conversation creation/deletion
- Space membership changes
- All stored in user-storage.dat

---

## What's Next?

- **[API Reference](./API_REFERENCE.md)**: Build integrations
- **[FAQ](./FAQ.md)**: Common questions
- **[Production Deployment](./PRODUCTION_DEPLOYMENT.md)**: Go live

---

**Last Updated**: October 26, 2025  
**Version**: 2.0.0
