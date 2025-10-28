# User Management System Documentation

**Complete guide to Sutra AI's authentication and session management system**

## Navigation

- [**System Architecture**](ARCHITECTURE.md) - Core components and data flow
- [**Registration Process**](REGISTRATION.md) - Complete user registration workflow
- [**Login Process**](LOGIN.md) - Authentication and session creation
- [**Session Management**](SESSION.md) - Session storage, validation, and lifecycle
- [**Logout Process**](LOGOUT.md) - Session invalidation and cleanup
- [**API Reference**](API.md) - Complete API endpoints and examples
- [**Storage Design**](STORAGE.md) - Vector-based user/session storage architecture
- [**Troubleshooting**](TROUBLESHOOTING.md) - Common issues and solutions

## Quick Reference

### Key Components
- **API Service**: FastAPI backend (`sutra-api`) on port 8000
- **User Storage**: Dedicated vector storage server (`user-storage-server`) on port 50053
- **Domain Storage**: Separate storage for business data (`storage-server`) on port 50051
- **Embedding Service**: HA embedding cluster on port 8888

### Authentication Flow
```
Registration → User Storage (vector) → Login → Session Creation → JWT Tokens → Logout
```

### Storage Strategy
- **User Data**: Vector embeddings only (no semantic analysis)
- **Business Data**: Full semantic analysis + vector embeddings
- **Sessions**: Pure vector storage for fast retrieval

## Critical Design Decisions (2025-10-28)

1. **Separate Storage Servers**: User authentication data isolated from business knowledge
2. **Vector-Only for Auth**: Disabled semantic analysis for user storage to prevent domain misclassification
3. **Session Embeddings**: Sessions require embeddings (`generate_embedding: true`) for proper storage
4. **Binary TCP Protocol**: Custom MessagePack protocol for high-performance storage access

## Recent Fixes Applied

- **Fixed Session Storage**: Sessions now properly stored with embeddings enabled
- **Disabled Semantic Analysis**: User storage configured with `SUTRA_SEMANTIC_ANALYSIS=false`
- **Vector Search Only**: Authentication uses pure vector search, not semantic classification
- **Race Condition**: Login includes delay to handle registration→login timing

## Getting Started

1. **Development Setup**: Ensure all services are running via `SUTRA_EDITION=simple ./sutra deploy`
2. **Test Registration**: `curl -X POST http://localhost:8000/auth/register -d '{"email":"test@example.com","password":"pass123","full_name":"Test User","organization":"Test Org"}'`
3. **Test Login**: `curl -X POST http://localhost:8000/auth/login -d '{"email":"test@example.com","password":"pass123"}'`
4. **Validate Session**: Use JWT tokens for authenticated requests

---

**Last Updated**: 2025-10-28  
**Version**: 2.0.0  
**AI Context**: This documentation includes complete technical details for AI systems to understand and maintain the user management architecture.