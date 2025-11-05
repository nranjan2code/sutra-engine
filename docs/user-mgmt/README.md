# User Management System Documentation

**Complete guide to Sutra AI's httpOnly cookie authentication and session management system**

## Navigation

- [**System Architecture**](ARCHITECTURE.md) - Core components and data flow
- [**Registration Process**](REGISTRATION.md) - Complete user registration workflow
- [**Login Process**](LOGIN.md) - Authentication and session creation (httpOnly cookies)
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
- **Security Middleware**: 8-layer OWASP headers + httpOnly cookie enforcement

### Authentication Flow
```
Registration → User Storage (vector) → Login → httpOnly Cookie Creation → JWT Tokens → Logout
```

### Storage Strategy
- **User Data**: Vector embeddings only (no semantic analysis)
- **Business Data**: Full semantic analysis + vector embeddings
- **Sessions**: Pure vector storage for fast retrieval
- **Tokens**: Server-side httpOnly cookies (NEVER localStorage - XSS immune)

## Critical Design Decisions (v3.0.0 - 2025-11-05)

1. **httpOnly Cookie Authentication**: Tokens stored server-side, never exposed to JavaScript (XSS immune)
2. **Separate Storage Servers**: User authentication data isolated from business knowledge
3. **Vector-Only for Auth**: Disabled semantic analysis for user storage to prevent domain misclassification
4. **Session Embeddings**: Sessions require embeddings (`generate_embedding: true`) for proper storage
5. **Binary TCP Protocol**: Custom MessagePack protocol for high-performance storage access (gRPC removed)
6. **Security Middleware**: 8-layer OWASP-compliant headers on every response

## Recent Fixes Applied (v3.0.0)

- **✅ httpOnly Cookies**: Removed ALL localStorage usage, tokens now in httpOnly cookies
- **✅ Security Headers**: HSTS, CSP, X-Frame-Options, X-Content-Type-Options, X-XSS-Protection, Referrer-Policy, Permissions-Policy
- **✅ Fixed Session Storage**: Sessions now properly stored with embeddings enabled
- **✅ Disabled Semantic Analysis**: User storage configured with `SUTRA_SEMANTIC_ANALYSIS=false`
- **✅ Vector Search Only**: Authentication uses pure vector search, not semantic classification
- **✅ Race Condition**: Login includes delay to handle registration→login timing
- **✅ gRPC Removed**: All legacy gRPC code deleted, TCP Binary Protocol only

## Getting Started

1. **Development Setup**: Ensure all services are running via `SUTRA_EDITION=simple ./sutra deploy`
2. **Test Registration**: `curl -X POST http://localhost:8000/auth/register -d '{"email":"test@example.com","password":"pass123","full_name":"Test User","organization":"Test Org"}'`
3. **Test Login**: `curl -X POST http://localhost:8000/auth/login -d '{"email":"test@example.com","password":"pass123"}'`
4. **Validate Session**: Use JWT tokens for authenticated requests

---

**Last Updated**: 2025-10-28  
**Version**: 2.0.0  
**AI Context**: This documentation includes complete technical details for AI systems to understand and maintain the user management architecture.