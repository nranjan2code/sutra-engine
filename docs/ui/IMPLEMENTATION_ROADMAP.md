# Conversation-First UI - Implementation Roadmap

REMEMBER THERE IS NO NEED FOR BACKWARD COMPATIBILITY 0 USERS EXIST

**Date:** October 26, 2025  
**Status:** Planning Phase  
**Version:** 1.0  

---

## 📋 Overview

This document provides a detailed, step-by-step implementation plan for the conversation-first UI architecture. The roadmap is designed for **multi-session implementation** with clear milestones, dependencies, and success criteria.

**Total Estimated Time:** 6-8 weeks  
**Team Size:** 1-2 developers  
**Complexity:** High (new architecture, dual storage, auth system)

---

## 🎯 Implementation Phases

```
Phase 1: Foundation (Week 1-2)
├─ User storage schema
├─ Authentication system
└─ Basic API endpoints

Phase 2: Core Chat (Week 3-4)
├─ Conversation management
├─ Message handling
└─ React chat interface

Phase 3: Advanced Features (Week 5-6)
├─ Spaces & organization
├─ Search functionality
└─ Permissions & RBAC

Phase 4: Polish & Deployment (Week 7-8)
├─ UI refinements
├─ Performance optimization
└─ Production deployment
```

---

## 📦 Phase 1: Foundation (Week 1-2)

### Goals

- Establish dual storage architecture
- Implement authentication system
- Create basic API structure

### Session 1.1: Storage Schema & Protocol (4-6 hours)

**Tasks:**

1. **Update Storage Protocol**
   - Location: `packages/sutra-protocol/src/messages.rs`
   - Add concept type discriminators (user, session, conversation, etc.)
   - Add metadata validation for new concept types
   
2. **Storage Schema Constants**
   - Location: `packages/sutra-storage/src/schema.rs` (new file)
   - Define concept type constants
   - Define association type constants
   - Define metadata field schemas

**Code Example:**

```rust
// packages/sutra-protocol/src/messages.rs

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConceptType {
    // Domain knowledge (existing)
    DomainConcept,
    
    // User management (new)
    User,
    Session,
    Conversation,
    Message,
    Space,
    Organization,
    Permission,
    AuditLog,
}

pub struct ConceptMetadata {
    pub concept_type: ConceptType,
    pub organization_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}
```

**Success Criteria:**

- [ ] Protocol supports all new concept types
- [ ] Metadata validation works correctly
- [ ] No breaking changes to existing domain storage
- [ ] Unit tests pass

**Estimated Time:** 4 hours

---

### Session 1.2: User Storage Deployment (2-3 hours)

**Tasks:**

1. **Docker Compose Configuration**
   - Location: `docker-compose-grid.yml`
   - Add `user-storage-server` service on port 50052
   - Add volume mount for `./data/user-storage`
   
2. **Deployment Script Updates**
   - Location: `sutra-deploy.sh`
   - Add user storage initialization
   - Add health check for user storage

**Code Example:**

```yaml
# docker-compose-grid.yml

services:
  user-storage-server:
    image: sutra-storage-server:${SUTRA_VERSION}
    container_name: user-storage-server
    ports:
      - "50052:50051"
    volumes:
      - ./data/user-storage:/data
    environment:
      - STORAGE_PATH=/data/user-storage.dat
      - SUTRA_EMBEDDING_SERVICE_URL=http://haproxy:8888
      - SUTRA_EDITION=${SUTRA_EDITION:-simple}
    networks:
      - sutra-network
    healthcheck:
      test: ["CMD", "grpc_health_probe", "-addr=:50051"]
      interval: 30s
      timeout: 10s
      retries: 3
```

**Success Criteria:**

- [ ] User storage server starts successfully
- [ ] Separate storage file created (`user-storage.dat`)
- [ ] Health checks pass
- [ ] No conflicts with existing storage

**Estimated Time:** 2 hours

---

### Session 1.3: Authentication API - Backend (6-8 hours)

**Tasks:**

1. **User Management Service**
   - Location: `packages/sutra-api/sutra_api/services/user_service.py` (new)
   - User registration
   - User login/logout
   - Password hashing (Argon2id)
   - Session management
   
2. **Auth Middleware**
   - Location: `packages/sutra-api/sutra_api/middleware/auth.py` (new)
   - JWT token generation
   - Token verification
   - Session validation
   
3. **Auth Endpoints**
   - Location: `packages/sutra-api/sutra_api/routes/auth.py` (new)
   - POST `/auth/register`
   - POST `/auth/login`
   - POST `/auth/logout`
   - GET `/auth/me`

**Code Example:**

```python
# packages/sutra-api/sutra_api/services/user_service.py

from argon2 import PasswordHasher
from datetime import datetime, timedelta
import jwt

class UserService:
    def __init__(self, user_storage_client):
        self.storage = user_storage_client
        self.ph = PasswordHasher()
        
    async def register(self, email: str, password: str, organization: str):
        # Check if user exists
        existing = await self.storage.semantic_search(
            f"user email:{email}",
            filters={"metadata.type": "user"}
        )
        if existing:
            raise ValueError("User already exists")
        
        # Hash password
        password_hash = self.ph.hash(password)
        
        # Create user concept
        user = await self.storage.create_concept(
            content=f"User {email}",
            semantic_patterns=["User"],
            metadata={
                "type": "user",
                "email": email,
                "password_hash": password_hash,
                "organization": organization,
                "role": "user",
                "active": True,
                "created_at": datetime.utcnow().isoformat()
            }
        )
        
        return user
    
    async def login(self, email: str, password: str):
        # Find user
        users = await self.storage.semantic_search(
            f"user email:{email}",
            filters={"metadata.type": "user", "metadata.active": True}
        )
        
        if not users:
            raise ValueError("Invalid credentials")
        
        user = users[0]
        
        # Verify password
        try:
            self.ph.verify(user.metadata["password_hash"], password)
        except:
            raise ValueError("Invalid credentials")
        
        # Create session
        session = await self.storage.create_concept(
            content=f"Session for {email}",
            semantic_patterns=["Session", "Active"],
            metadata={
                "type": "session",
                "user_id": user.id,
                "created_at": datetime.utcnow().isoformat(),
                "expires_at": (datetime.utcnow() + timedelta(hours=24)).isoformat(),
                "active": True
            }
        )
        
        # Create association
        await self.storage.create_association(
            user.id, session.id, "has_session"
        )
        
        # Generate JWT
        token = jwt.encode({
            "user_id": user.id,
            "session_id": session.id,
            "exp": datetime.utcnow() + timedelta(hours=24)
        }, "SECRET_KEY_FROM_ENV")
        
        return {"token": token, "user": user}
```

**Success Criteria:**

- [ ] User registration works
- [ ] Login creates session in user storage
- [ ] JWT tokens generated correctly
- [ ] Password hashing secure (Argon2id)
- [ ] Session validation works
- [ ] Unit tests for all flows

**Estimated Time:** 6 hours

---

### Session 1.4: Authentication API - Frontend (4-6 hours)

**Tasks:**

1. **Auth Context**
   - Location: `packages/sutra-client/src/contexts/AuthContext.tsx` (new)
   - Login/logout functions
   - httpOnly Cookie Authentication (v3.0.0)
   - Tokens stored server-side (XSS immune)
   - Automatic cookie management by browser
   - NO localStorage usage (security vulnerability)
   - User state management
   
2. **Login Page**
   - Location: `packages/sutra-client/src/pages/Login.tsx` (new)
   - Email/password form
   - Error handling
   - Redirect after login
   
3. **Protected Route Wrapper**
   - Location: `packages/sutra-client/src/components/ProtectedRoute.tsx` (new)
   - Check authentication
   - Redirect to login if needed

**Code Example:**

```tsx
// packages/sutra-client/src/contexts/AuthContext.tsx

import React, { createContext, useState, useContext, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import api from '../api/client';

interface User {
  id: string;
  email: string;
  role: string;
  organization: string;
}

interface AuthContextType {
  user: User | null;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  loading: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();
  
  useEffect(() => {
    // Check if user is logged in
    // v3.0.0: httpOnly cookies (XSS immune)
    // Tokens automatically sent via Cookie header
    // NO manual token management needed!
    
    // Authentication happens automatically via cookies
    if (token) {
      api.get('/auth/me')
        .then(response => setUser(response.data))
        .catch(() => {
          // v3.0.0: Server clears httpOnly cookies
          // NO client-side token management needed
        })
        .finally(() => setLoading(false));
    } else {
      setLoading(false);
    }
  }, []);
  
  const login = async (email: string, password: string) => {
    const response = await api.post('/auth/login', { email, password });
    // v3.0.0: Server sets httpOnly cookies automatically
    // Response contains user data only (NO tokens in response body)
    // Cookies sent automatically with every subsequent request
    setUser(response.data.user);
    navigate('/');
  };
  
  const logout = async () => {
    await api.post('/auth/logout');
    // v3.0.0: Server invalidates session and clears httpOnly cookies
    // Client just needs to call logout endpoint
    setUser(null);
    navigate('/login');
  };
  
  return (
    <AuthContext.Provider value={{ user, login, logout, loading }}>
      {children}
    </AuthContext.Provider>
  );
}

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) throw new Error('useAuth must be used within AuthProvider');
  return context;
};
```

**Success Criteria:**

- [ ] Login page functional
- [x] httpOnly cookies (v3.0.0 - XSS immune, server-managed)
- [x] Automatic cookie management (browser handles send/receive)
- [x] CSRF protection (SameSite=lax cookies)
- [ ] Protected routes work
- [ ] Logout clears session
- [ ] Auto-redirect to login when not authenticated

**Estimated Time:** 4 hours

---

**Phase 1 Total:** 16-23 hours (2-3 days)

---

## 💬 Phase 2: Core Chat (Week 3-4)

### Goals

- Implement conversation management
- Build chat interface
- Message streaming

### Session 2.1: Conversation Service (6-8 hours)

**Tasks:**

1. **Conversation Management Service**
   - Location: `packages/sutra-api/sutra_api/services/conversation_service.py` (new)
   - Create conversation
   - Load conversation history
   - Add message to conversation
   - Update conversation metadata
   
2. **Chat Endpoints**
   - Location: `packages/sutra-api/sutra_api/routes/conversations.py` (new)
   - POST `/conversations/create`
   - GET `/conversations/list`
   - GET `/conversations/{id}/messages`
   - POST `/conversations/{id}/message`

**Code Example:**

```python
# packages/sutra-api/sutra_api/services/conversation_service.py

class ConversationService:
    def __init__(self, user_storage_client, domain_storage_clients):
        self.user_storage = user_storage_client
        self.domain_storages = domain_storage_clients
    
    async def create_conversation(
        self, 
        user_id: str, 
        space_id: str,
        organization: str,
        domain_storage: str
    ):
        # Create conversation concept
        conv = await self.user_storage.create_concept(
            content="New conversation",
            semantic_patterns=["Conversation"],
            metadata={
                "type": "conversation",
                "user_id": user_id,
                "space_id": space_id,
                "organization": organization,
                "domain_storage": domain_storage,
                "message_count": 0,
                "created_at": datetime.utcnow().isoformat()
            }
        )
        
        # Create associations
        await self.user_storage.create_association(
            user_id, conv.id, "owns_conversation"
        )
        await self.user_storage.create_association(
            space_id, conv.id, "contains_conversation"
        )
        
        return conv
    
    async def send_message(
        self,
        conversation_id: str,
        user_id: str,
        message: str
    ):
        # Load conversation
        conv = await self.user_storage.get_concept(conversation_id)
        
        # Create user message
        user_msg = await self.user_storage.create_concept(
            content=message,
            semantic_patterns=["UserMessage"],
            metadata={
                "type": "user_message",
                "conversation_id": conversation_id,
                "user_id": user_id,
                "timestamp": datetime.utcnow().isoformat()
            }
        )
        
        # Load context (last 10 messages)
        context = await self.load_context(conversation_id, limit=10)
        
        # Query domain storage
        domain_storage_name = conv.metadata["domain_storage"]
        domain_storage = self.domain_storages[domain_storage_name]
        
        reasoning_result = await domain_storage.reason(
            query=message,
            context=context
        )
        
        # Create assistant message
        assistant_msg = await self.user_storage.create_concept(
            content=reasoning_result.answer,
            semantic_patterns=["AssistantMessage", "Reasoning"],
            metadata={
                "type": "assistant_message",
                "conversation_id": conversation_id,
                "timestamp": datetime.utcnow().isoformat(),
                "reasoning_paths": reasoning_result.paths,
                "concepts_used": reasoning_result.concept_ids,
                "confidence": reasoning_result.confidence
            }
        )
        
        # Create associations
        await self.user_storage.create_association(
            conversation_id, user_msg.id, "contains_message"
        )
        await self.user_storage.create_association(
            conversation_id, assistant_msg.id, "contains_message"
        )
        
        # Update conversation
        await self.user_storage.update_concept_metadata(
            conversation_id,
            {"message_count": conv.metadata["message_count"] + 2}
        )
        
        return assistant_msg
```

**Success Criteria:**

- [ ] Conversations created in user storage
- [ ] Messages linked to conversations
- [ ] Context loading works
- [ ] Domain storage queried correctly
- [ ] Associations maintained

**Estimated Time:** 6 hours

---

### Session 2.2: Chat UI - Basic Layout (6-8 hours)

**Tasks:**

1. **Sidebar Component**
   - Location: `packages/sutra-client/src/components/Sidebar.tsx`
   - Conversation list
   - Space selector
   - Collapsible design
   
2. **Chat Interface Component**
   - Location: `packages/sutra-client/src/components/ChatInterface.tsx`
   - Message list
   - Message input
   - Send button
   
3. **Message Component**
   - Location: `packages/sutra-client/src/components/Message.tsx`
   - User message display
   - Assistant message display
   - Timestamp, avatar

**Code Example:**

```tsx
// packages/sutra-client/src/components/ChatInterface.tsx

import React, { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '../api/client';
import Message from './Message';
import MessageInput from './MessageInput';

export default function ChatInterface() {
  const { conversationId } = useParams();
  const queryClient = useQueryClient();
  
  // Load messages
  const { data: messages, isLoading } = useQuery({
    queryKey: ['messages', conversationId],
    queryFn: () => api.get(`/conversations/${conversationId}/messages`),
    enabled: !!conversationId
  });
  
  // Send message mutation
  const sendMessage = useMutation({
    mutationFn: (message: string) =>
      api.post(`/conversations/${conversationId}/message`, { message }),
    onSuccess: (response) => {
      queryClient.invalidateQueries(['messages', conversationId]);
    }
  });
  
  return (
    <div className="flex flex-col h-full">
      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {isLoading ? (
          <div>Loading...</div>
        ) : (
          messages?.map(msg => (
            <Message key={msg.id} message={msg} />
          ))
        )}
      </div>
      
      {/* Input */}
      <MessageInput
        onSend={sendMessage.mutate}
        isLoading={sendMessage.isLoading}
      />
    </div>
  );
}
```

**Success Criteria:**

- [ ] Sidebar shows conversations
- [ ] Chat interface loads messages
- [ ] Message input functional
- [ ] Responsive design
- [ ] Loading states handled

**Estimated Time:** 6 hours

---

### Session 2.3: Message Streaming (4-6 hours)

**Tasks:**

1. **Server-Sent Events (SSE) Backend**
   - Location: `packages/sutra-api/sutra_api/routes/conversations.py`
   - Stream reasoning progress
   - Progressive refinement
   
2. **SSE Frontend**
   - Location: `packages/sutra-client/src/hooks/useMessageStream.ts`
   - EventSource connection
   - Incremental message updates

**Success Criteria:**

- [ ] Messages stream progressively
- [ ] Confidence updates shown
- [ ] Smooth UX (no jarring updates)

**Estimated Time:** 5 hours

---

**Phase 2 Total:** 16-22 hours (2-3 days)

---

## 🏢 Phase 3: Advanced Features (Week 5-6)

### Session 3.1: Spaces & Organization (6-8 hours)

**Tasks:**

1. **Space Management Service**
2. **Space UI Components**
3. **Space Permissions (RBAC)**

**Estimated Time:** 7 hours

---

### Session 3.2: Search Functionality (6-8 hours)

**Tasks:**

1. **Semantic Search Backend**
2. **Search UI Component**
3. **Search Results Display**

**Estimated Time:** 7 hours

---

### Session 3.3: Graph Visualization (8-10 hours)

**Tasks:**

1. **Graph Data API**
2. **D3.js Integration**
3. **Interactive Graph Component**

**Estimated Time:** 9 hours

---

**Phase 3 Total:** 20-26 hours (3-4 days)

---

## 🎨 Phase 4: Polish & Deployment (Week 7-8)

### Session 4.1: UI/UX Refinements (8-10 hours)
### Session 4.2: Performance Optimization (6-8 hours)
### Session 4.3: Production Deployment (4-6 hours)
### Session 4.4: Documentation & Testing (6-8 hours)

**Phase 4 Total:** 24-32 hours (3-4 days)

---

## 📊 Summary

| Phase | Duration | Hours | Key Deliverables |
|-------|----------|-------|------------------|
| **Phase 1** | Week 1-2 | 16-23h | Auth system, dual storage, API foundation |
| **Phase 2** | Week 3-4 | 16-22h | Chat interface, conversations, messaging |
| **Phase 3** | Week 5-6 | 20-26h | Spaces, search, graph visualization |
| **Phase 4** | Week 7-8 | 24-32h | Polish, optimization, deployment |
| **Total** | 6-8 weeks | 76-103h | Complete conversation-first UI |

---

## 🎯 Success Metrics

### Technical Metrics

- [ ] All API endpoints operational
- [ ] <100ms p99 latency for chat messages
- [ ] <50ms p99 latency for authentication
- [ ] Zero data loss (conversations persisted correctly)
- [ ] 100% test coverage for critical paths

### User Experience Metrics

- [ ] Can create account and log in
- [ ] Can start conversation and get responses
- [ ] Can search past conversations
- [ ] Can navigate graph visualization
- [ ] Can manage spaces and permissions

### Quality Metrics

- [ ] WCAG 2.1 AA accessibility compliance
- [ ] Mobile responsive (tablet+)
- [ ] No console errors
- [ ] Proper error handling
- [ ] Loading states for all async operations

---

## 🚀 Next Steps

1. Review this roadmap with team
2. Set up project tracking (GitHub Issues/Projects)
3. Begin with Phase 1, Session 1.1
4. Schedule regular check-ins (every 2-3 sessions)
5. Update TODO.md as sessions complete

---

**Ready to start implementation!**
