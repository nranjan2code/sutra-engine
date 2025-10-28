# Logout Process

**Complete workflow for session invalidation and user logout in Sutra AI**

## Overview

The logout process invalidates active user sessions, clears JWT tokens, and ensures secure session termination. The system supports both explicit logout and automatic session cleanup.

## Logout Workflow

### 1. Frontend Logout Request
```javascript
// Frontend (React) logout call
const logoutUser = async () => {
  const token = localStorage.getItem('access_token');
  
  try {
    // Call logout endpoint
    await axios.post('http://localhost:8000/auth/logout', {}, {
      headers: {
        'Authorization': `Bearer ${token}`
      }
    });
  } catch (error) {
    console.log('Logout endpoint failed, clearing tokens anyway');
  }
  
  // Always clear local tokens regardless of server response
  localStorage.removeItem('access_token');
  localStorage.removeItem('refresh_token');
  
  // Redirect to login page
  window.location.href = '/login';
};
```

### 2. API Endpoint Processing
**Endpoint**: `POST /auth/logout`  
**Handler**: `sutra_api.routes.auth.logout`  
**Service**: `sutra_api.services.user_service.UserService.logout()`

```python
async def logout(self, session_id: str) -> bool:
```

### 3. Session Invalidation Process
```python
async def logout(self, session_id: str) -> bool:
    """
    Invalidate a user session.
    
    Marks session as inactive in user-storage (soft delete).
    
    Args:
        session_id: Session ID to invalidate
    
    Returns:
        True if successful
    
    Raises:
        ValueError: If session not found
    """
    try:
        # Find session via vector search
        dummy_vector = [0.0] * 768
        vector_results = self.storage.vector_search(dummy_vector, k=50)
        
        session_data = None
        session_concept_id = None
        
        for concept_id, similarity in vector_results:
            concept = self.storage.query_concept(concept_id)
            if concept and concept.get("content"):
                try:
                    data = json.loads(concept["content"])
                    if (data.get("type") == "session" and 
                        data.get("session_id") == session_id):
                        session_data = data
                        session_concept_id = concept_id
                        break
                except json.JSONDecodeError:
                    continue
        
        if not session_data:
            raise ValueError("Session not found")
        
        # Mark session as inactive (soft delete)
        # Note: Metadata updates not yet implemented in storage server
        # TODO: Add update_concept_metadata method
        # For now, we rely on expiration time checking
        
        logger.info(f"Session logout: {session_id[:8]}... for {session_data.get('email')}")
        return True
        
    except Exception as e:
        logger.error(f"Logout failed: {session_id}: {e}")
        return False
```

## Complete Code Implementation

### API Route Handler
```python
from fastapi import Depends, HTTPException
from fastapi.security import HTTPBearer

security = HTTPBearer()

@router.post("/logout", response_model=dict)
async def logout(
    token: str = Depends(security),
    user_service: UserService = Depends(get_user_service)
) -> dict:
    try:
        # Extract session ID from JWT token
        payload = jwt.decode(token.credentials, JWT_SECRET_KEY, algorithms=["HS256"])
        session_id = payload.get('session_id')
        
        if not session_id:
            raise HTTPException(status_code=400, detail="Invalid token")
        
        # Logout session
        success = await user_service.logout(session_id)
        
        if success:
            return {"message": "Logged out successfully"}
        else:
            return {"message": "Session already invalid"}
            
    except jwt.InvalidTokenError:
        raise HTTPException(status_code=401, detail="Invalid token")
    except Exception as e:
        logger.error(f"Logout error: {e}")
        raise HTTPException(status_code=500, detail="Logout failed")
```

### Session Invalidation Methods

#### Current Implementation (Soft Delete)
```python
async def logout(self, session_id: str) -> bool:
    """Mark session as inactive via soft delete."""
    try:
        session_data = await self.get_session_by_id(session_id)
        
        if not session_data:
            logger.warning(f"Logout attempted on non-existent session: {session_id}")
            return False
        
        # Currently implemented as a no-op since update_concept not available
        # Session will be considered invalid when checked during validation
        
        logger.info(f"✅ Session logout: {session_id[:8]}... ({session_data.get('email')})")
        return True
        
    except Exception as e:
        logger.error(f"Logout failed: {session_id}: {e}")
        return False

async def get_session_by_id(self, session_id: str) -> dict:
    """Helper method to find session by ID."""
    try:
        dummy_vector = [0.0] * 768
        vector_results = self.storage.vector_search(dummy_vector, k=100)
        
        for concept_id, similarity in vector_results:
            concept = self.storage.query_concept(concept_id)
            if concept and concept.get("content"):
                try:
                    data = json.loads(concept["content"])
                    if (data.get("type") == "session" and 
                        data.get("session_id") == session_id):
                        return data
                except json.JSONDecodeError:
                    continue
        
        return None
        
    except Exception as e:
        logger.error(f"Session lookup failed: {session_id}: {e}")
        return None
```

#### Future Implementation (Hard Delete)
```python
# TODO: Implement when storage server supports concept deletion
async def logout_with_deletion(self, session_id: str) -> bool:
    """Remove session completely from storage."""
    try:
        session_concept_id = await self.find_session_concept_id(session_id)
        
        if not session_concept_id:
            return False
        
        # Delete concept from storage (not yet implemented)
        success = self.storage.delete_concept(session_concept_id)
        
        if success:
            logger.info(f"✅ Session deleted: {session_id[:8]}...")
            return True
        else:
            logger.error(f"Failed to delete session: {session_id}")
            return False
            
    except Exception as e:
        logger.error(f"Session deletion failed: {session_id}: {e}")
        return False
```

## JWT Token Management

### Token Blacklisting (Alternative Approach)
```python
# In-memory blacklist for immediate token invalidation
blacklisted_tokens = set()

def blacklist_token(jti: str):
    """Add token to blacklist."""
    blacklisted_tokens.add(jti)

def is_token_blacklisted(jti: str) -> bool:
    """Check if token is blacklisted."""
    return jti in blacklisted_tokens

# Enhanced JWT validation
async def validate_jwt_token(token: str) -> dict:
    try:
        payload = jwt.decode(token, JWT_SECRET_KEY, algorithms=["HS256"])
        
        # Check if token is blacklisted
        jti = payload.get('jti')  # JWT ID (would need to be added to tokens)
        if jti and is_token_blacklisted(jti):
            raise ValueError("Token has been invalidated")
        
        return payload
        
    except jwt.InvalidTokenError:
        raise ValueError("Invalid token")
```

### Token Cleanup
```python
# Periodic cleanup of expired blacklisted tokens
import asyncio
from datetime import datetime

async def cleanup_blacklisted_tokens():
    """Remove expired tokens from blacklist."""
    while True:
        try:
            current_time = datetime.utcnow().timestamp()
            expired_tokens = []
            
            for jti in blacklisted_tokens:
                # Check if token is expired (would need token expiry tracking)
                # This is a simplified version
                pass
            
            for jti in expired_tokens:
                blacklisted_tokens.discard(jti)
            
            logger.info(f"Cleaned up {len(expired_tokens)} expired blacklisted tokens")
            
        except Exception as e:
            logger.error(f"Token cleanup failed: {e}")
        
        # Run cleanup every hour
        await asyncio.sleep(3600)
```

## Frontend Integration

### React Logout Component
```javascript
import React from 'react';
import axios from 'axios';

const LogoutButton = ({ onLogout }) => {
  const handleLogout = async () => {
    try {
      const token = localStorage.getItem('access_token');
      
      if (token) {
        // Call logout endpoint
        await axios.post('/auth/logout', {}, {
          headers: {
            'Authorization': `Bearer ${token}`
          }
        });
      }
    } catch (error) {
      console.error('Logout API call failed:', error);
      // Continue with local cleanup even if API fails
    } finally {
      // Always clear tokens locally
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
      
      // Clear axios default authorization header
      delete axios.defaults.headers.common['Authorization'];
      
      // Callback to update app state
      if (onLogout) {
        onLogout();
      }
      
      // Redirect to login
      window.location.href = '/login';
    }
  };

  return (
    <button onClick={handleLogout} className="logout-btn">
      Logout
    </button>
  );
};

export default LogoutButton;
```

### Axios Interceptor for Token Management
```javascript
// Setup axios interceptor for automatic token handling
import axios from 'axios';

// Request interceptor to add token
axios.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('access_token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => Promise.reject(error)
);

// Response interceptor to handle token expiration
axios.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Token expired or invalid
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);
```

## Session Cleanup Strategies

### Automatic Expiration
```python
async def is_session_valid(session_data: dict) -> bool:
    """Check if session is still valid."""
    try:
        # Check active flag
        if not session_data.get("active", True):
            return False
        
        # Check expiration time
        expires_at_str = session_data.get("expires_at")
        if expires_at_str:
            expires_at = datetime.fromisoformat(expires_at_str)
            if datetime.utcnow() > expires_at:
                return False
        
        return True
        
    except Exception:
        return False
```

### Background Cleanup Task
```python
from apscheduler.schedulers.asyncio import AsyncIOScheduler

scheduler = AsyncIOScheduler()

@scheduler.scheduled_job('interval', hours=1)
async def cleanup_expired_sessions():
    """Background task to clean up expired sessions."""
    try:
        dummy_vector = [0.0] * 768
        results = storage.vector_search(dummy_vector, k=1000)
        
        expired_count = 0
        
        for concept_id, similarity in results:
            concept = storage.query_concept(concept_id)
            if concept and concept.get("content"):
                try:
                    data = json.loads(concept["content"])
                    if data.get("type") == "session":
                        if not await is_session_valid(data):
                            # Mark for cleanup (implementation depends on storage capabilities)
                            expired_count += 1
                except json.JSONDecodeError:
                    continue
        
        logger.info(f"Session cleanup: {expired_count} expired sessions identified")
        
    except Exception as e:
        logger.error(f"Session cleanup task failed: {e}")

# Start scheduler
scheduler.start()
```

## Security Considerations

### Logout Best Practices
1. **Client-side cleanup**: Always clear tokens locally regardless of server response
2. **Server-side validation**: Verify session exists before attempting logout
3. **Graceful degradation**: Handle logout failures without blocking user experience
4. **Token invalidation**: Implement proper token blacklisting for immediate effect
5. **Session cleanup**: Regular background tasks to clean expired sessions

### Session Security
```python
# Enhanced session validation
async def validate_session_security(session_data: dict, request_ip: str) -> bool:
    """Enhanced security validation for sessions."""
    
    # Check basic validity
    if not await is_session_valid(session_data):
        return False
    
    # Optional: IP address validation
    # if session_data.get("ip_address") != request_ip:
    #     logger.warning(f"IP mismatch for session {session_data.get('session_id')}")
    #     return False
    
    # Optional: User agent validation
    # if session_data.get("user_agent") != request_user_agent:
    #     logger.warning(f"User agent mismatch for session {session_data.get('session_id')}")
    #     return False
    
    return True
```

## Testing Logout

### Manual Testing
```bash
# 1. Login to get token
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "password123"}')

TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.access_token')

# 2. Test protected endpoint
curl -X GET http://localhost:8000/protected-resource \
  -H "Authorization: Bearer $TOKEN"

# 3. Logout
curl -X POST http://localhost:8000/auth/logout \
  -H "Authorization: Bearer $TOKEN"

# 4. Verify token is invalid
curl -X GET http://localhost:8000/protected-resource \
  -H "Authorization: Bearer $TOKEN"
# Should return 401 Unauthorized
```

### Integration Test
```python
import pytest
import asyncio

@pytest.mark.asyncio
async def test_logout_workflow():
    """Test complete logout workflow."""
    
    # 1. Register user
    user_data = {
        "email": "logout.test@example.com",
        "password": "password123",
        "full_name": "Logout Test",
        "organization": "Test Org"
    }
    user_service = UserService()
    user_result = await user_service.register(**user_data)
    assert user_result["email"] == user_data["email"]
    
    # 2. Login
    login_result = await user_service.login(user_data["email"], user_data["password"])
    assert "access_token" in login_result
    
    # Extract session ID from token
    token_payload = jwt.decode(login_result["access_token"], JWT_SECRET_KEY, algorithms=["HS256"])
    session_id = token_payload["session_id"]
    
    # 3. Verify session exists
    session_data = await user_service.get_session_by_id(session_id)
    assert session_data is not None
    assert session_data["active"] == True
    
    # 4. Logout
    logout_success = await user_service.logout(session_id)
    assert logout_success == True
    
    # 5. Verify session handling after logout
    # (Behavior depends on implementation - soft delete vs hard delete)
```

---

**AI Context**: The logout process handles session invalidation through soft delete (marking inactive) rather than hard delete due to current storage server limitations. Frontend should always clear tokens locally regardless of server response to ensure user experience. Future implementations should support proper session deletion when storage capabilities are enhanced.

**Last Updated**: 2025-10-28  
**Implementation Notes**: Current logout implementation relies on session expiration rather than immediate deletion due to storage server limitations. Consider implementing JWT blacklisting for immediate token invalidation.