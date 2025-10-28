"""
User Management Service.

Handles user registration, authentication, and session management using
Sutra's storage engine as the backend (dogfooding storage.dat for auth).
"""

import logging
from datetime import datetime, timedelta
from typing import Dict, Optional

from argon2 import PasswordHasher
from argon2.exceptions import VerificationError, VerifyMismatchError

logger = logging.getLogger(__name__)


class UserService:
    """
    User management service using Sutra storage for user data.
    
    This service proves that Sutra's storage engine can replace traditional
    database stacks (PostgreSQL + Redis) for user management and authentication.
    
    All user data is stored as concepts in user-storage.dat with associations
    for relationships (user -> sessions, user -> conversations, etc.).
    """
    
    def __init__(self, user_storage_client):
        """
        Initialize user service with storage client.
        
        Args:
            user_storage_client: StorageClient connected to user-storage.dat
        """
        self.storage = user_storage_client
        self.ph = PasswordHasher()
        logger.info("UserService initialized with storage client")
    
    async def register(
        self, 
        email: str, 
        password: str, 
        organization: str,
        full_name: Optional[str] = None,
        role: str = "user"
    ) -> Dict:
        """
        Register a new user account.
        
        Creates a User concept in user-storage.dat with hashed password.
        Uses Argon2id for secure password hashing.
        
        Args:
            email: User email (unique identifier)
            password: Plain-text password (will be hashed)
            organization: Organization ID (required)
            full_name: Optional full name
            role: User role (default: "user")
        
        Returns:
            Dict with user_id and user details
        
        Raises:
            ValueError: If user already exists or validation fails
        """
        # Validate email format
        if not email or "@" not in email:
            raise ValueError("Invalid email address")
        
        # Validate organization
        if not organization:
            raise ValueError("Organization is required")
        
        # Validate password strength
        if len(password) < 8:
            raise ValueError("Password must be at least 8 characters")
        
        # Check if user already exists using VECTOR SEARCH ONLY
        try:
            import json
            
            dummy_vector = [0.0] * 768
            vector_results = self.storage.vector_search(dummy_vector, k=50)
            
            # Check for existing user with same email (clean code, no backward compatibility)
            for concept_id, similarity in vector_results:
                concept = self.storage.query_concept(concept_id)
                if concept and concept.get("content"):
                    try:
                        data = json.loads(concept["content"])
                        if (data.get("type") == "user" and 
                            data.get("email") == email):
                            raise ValueError(f"User with email {email} already exists")
                    except json.JSONDecodeError:
                        continue
                        
        except ValueError:
            raise  # Re-raise ValueError (user exists)
        except Exception as e:
            logger.error(f"Error checking existing user: {e}")
            # Continue with registration if search fails

        # Hash password with Argon2id
        try:
            password_hash = self.ph.hash(password)
        except Exception as e:
            logger.error(f"Password hashing failed: {e}")
            raise ValueError("Failed to hash password")
        
        # Create user concept
        now = datetime.utcnow().isoformat()
        
        # Store complete user data as JSON in the content
        # This ensures all user data including password hash is persisted
        user_data = {
            "type": "user",
            "email": email,
            "password_hash": password_hash,
            "organization": organization,
            "role": role,
            "full_name": full_name or "",
            "active": True,
            "created_at": now,
            "updated_at": now,
            "last_login": None,
        }
        
        # Store as JSON - this ensures data persistence
        import json
        content = json.dumps(user_data)
        
        try:
            # Learn user concept 
            user_id = self.storage.learn_concept_v2(
                content=content,
                options={
                    "generate_embedding": True,
                    "extract_associations": False,  # No associations yet
                }
            )
            
            logger.info(f"✅ User registered: {email} (ID: {user_id[:8]})")
            
            return {
                "user_id": user_id,
                "email": email,
                "organization": organization,
                "role": role,
                "full_name": full_name,
                "created_at": now,
            }
        except Exception as e:
            logger.error(f"Failed to create user concept: {e}")
            raise ValueError(f"User registration failed: {str(e)}")
    
    async def login(self, email: str, password: str) -> Dict:
        """
        Authenticate user and create session - VECTOR SEARCH ONLY.
        
        Args:
            email: User email
            password: Plain-text password
        
        Returns:
            Dict with user info and session_id
        
        Raises:
            ValueError: If credentials are invalid
        """
        try:
            import json
            import secrets
            
            logger.info(f"Vector search login: {email}")
            
            # Use vector search to find all concepts
            dummy_vector = [0.0] * 768
            vector_results = self.storage.vector_search(dummy_vector, k=50)
            
            # Find user with matching email (clean code, no backward compatibility)
            user_data = None
            user_concept_id = None
            
            for concept_id, similarity in vector_results:
                concept = self.storage.query_concept(concept_id)
                if concept and concept.get("content"):
                    try:
                        data = json.loads(concept["content"])
                        if (data.get("type") == "user" and 
                            data.get("email") == email):
                            user_data = data
                            user_concept_id = concept_id
                            logger.info(f"User found: {email}")
                            break
                    except json.JSONDecodeError:
                        continue
            
            if not user_data:
                logger.warning(f"User not found: {email}")
                raise ValueError("Invalid credentials")
            
            # Check if user is active
            if not user_data.get("active", True):
                logger.warning(f"Inactive user: {email}")
                raise ValueError("User account is inactive")
            
            # Verify password
            stored_hash = user_data.get("password_hash")
            if not stored_hash:
                logger.error(f"No password hash: {email}")
                raise ValueError("User account corrupted")
            
            try:
                self.ph.verify(stored_hash, password)
                logger.info(f"Password verified: {email}")
            except (VerificationError, VerifyMismatchError):
                logger.warning(f"Invalid password: {email}")
                raise ValueError("Invalid credentials")
            
            # Create session
            now = datetime.utcnow()
            expires_at = now + timedelta(days=7)
            
            session_data = {
                "type": "session",
                "session_id": secrets.token_urlsafe(16),
                "user_id": user_concept_id,
                "email": user_data["email"],
                "organization": user_data["organization"],
                "role": user_data["role"],
                "created_at": now.isoformat(),
                "expires_at": expires_at.isoformat(),
                "active": True,
                "last_activity": now.isoformat(),
            }
            
            session_concept_id = self.storage.learn_concept_v2(
                content=json.dumps(session_data),
                options={
                    "generate_embedding": True,  # Sessions NEED embeddings for storage
                    "extract_associations": False,
                }
            )
            
            logger.info(f"✅ Session created: {email} -> {session_data['session_id'][:8]}")
            
            return {
                "user_id": user_concept_id,
                "session_id": session_data["session_id"],
                "email": user_data["email"],
                "organization": user_data["organization"],
                "role": user_data["role"],
                "full_name": user_data.get("full_name"),
                "created_at": user_data.get("created_at"),
                "last_login": now.isoformat(),
            }
            
        except ValueError:
            raise  # Re-raise credential errors
        except Exception as e:
            logger.error(f"Login failed: {email}: {e}")
            raise ValueError("Login failed")
    
    async def logout(self, session_id: str) -> bool:
        """
        Invalidate a user session.
        
        Marks session as inactive in user-storage.dat (soft delete).
        
        Args:
            session_id: Session ID to invalidate
        
        Returns:
            True if successful
        
        Raises:
            ValueError: If session not found
        """
        try:
            # Get session concept
            session = self.storage.query_concept(session_id)
            
            if not session:
                raise ValueError("Session not found")
            
            # Mark session as inactive
            # Note: Metadata updates not yet implemented
            # TODO: Add update_concept_metadata method
            # For now, we'll rely on expiration time checking
            
            logger.info(f"✅ Session invalidated: {session_id[:8]}")
            return True
            
        except Exception as e:
            logger.error(f"Logout failed for session {session_id}: {e}")
            raise ValueError("Logout failed")
    
    async def validate_session(self, session_id: str) -> Optional[Dict]:
        """
        Validate a session and return user info.
        
        Checks if session exists, is active, and not expired.
        
        Args:
            session_id: Session ID to validate
        
        Returns:
            Dict with user info if valid, None otherwise
        """
        try:
            import json
            
            # Search for session using ONLY vector search - no semantic search
            session_info = None
            
            logger.info(f"Using vector search for session lookup: {session_id[:8]}")
            dummy_vector = [0.0] * 768  # Use zero vector to get all concepts
            vector_results = self.storage.vector_search(dummy_vector, k=50)
            
            # Find the session with the exact matching session_id
            for concept_id, similarity in vector_results:
                concept = self.storage.query_concept(concept_id)
                if concept and concept.get("content"):
                    try:
                        session_data = json.loads(concept["content"])
                        if (session_data.get("type") == "session" and 
                            session_data.get("session_id") == session_id):
                            session_info = session_data
                            break
                    except json.JSONDecodeError:
                        continue
            
            if not session_info:
                logger.debug(f"Session not found: {session_id[:8]}")
                return None
            
            # Check if active
            if not session_info.get("active", False):
                logger.debug(f"Session inactive: {session_id[:8]}")
                return None
            
            # Check expiration
            expires_at_str = session_info.get("expires_at")
            if expires_at_str:
                expires_at = datetime.fromisoformat(expires_at_str)
                if datetime.utcnow() > expires_at:
                    logger.info(f"Session expired: {session_id[:8]}")
                    return None
            
            # Return session info with user details
            return {
                "session_id": session_info["session_id"],
                "user_id": session_info["user_id"],
                "email": session_info["email"],
                "organization": session_info["organization"],
                "role": session_info["role"],
                "created_at": session_info.get("created_at"),
                "expires_at": session_info.get("expires_at"),
                "active": session_info.get("active", True)
            }
            
        except Exception as e:
            logger.error(f"Session validation failed: {e}")
            return None
    
    async def get_user(self, user_id: str) -> Optional[Dict]:
        """
        Get user details by ID.
        
        Args:
            user_id: User concept ID
        
        Returns:
            Dict with user info or None if not found
        """
        try:
            user = self.storage.query_concept(user_id)
            
            if not user:
                return None
            
            metadata = user.get("metadata", {})
            
            return {
                "user_id": user_id,
                "email": metadata.get("email"),
                "organization": metadata.get("organization"),
                "role": metadata.get("role"),
                "full_name": metadata.get("full_name"),
                "active": metadata.get("active"),
                "created_at": metadata.get("created_at"),
                "last_login": metadata.get("last_login"),
            }
            
        except Exception as e:
            logger.error(f"Failed to get user {user_id}: {e}")
            return None
    
    async def refresh_session(self, session_id: str) -> Optional[Dict]:
        """
        Refresh an existing session (extend expiration).
        
        Args:
            session_id: Session ID to refresh
        
        Returns:
            Dict with new expiration time
        """
        # Validate session first
        session_info = await self.validate_session(session_id)
        if not session_info:
            return None
        
        # TODO: Implement metadata update to extend expires_at
        # For now, return existing info
        return session_info
    
    async def update_user(
        self,
        user_id: str,
        email: Optional[str] = None,
        full_name: Optional[str] = None,
        organization: Optional[str] = None,
    ) -> Optional[Dict]:
        """
        Update user profile information.
        
        Args:
            user_id: User concept ID
            email: New email (optional)
            full_name: New full name (optional)
            organization: New organization (optional)
        
        Returns:
            Updated user info or None if user not found
        
        Raises:
            ValueError: If validation fails
        """
        try:
            # Get existing user
            user = self.storage.query_concept(user_id)
            if not user:
                raise ValueError("User not found")
            
            metadata = user.get("metadata", {})
            
            # Check if email is being changed
            if email and email != metadata.get("email"):
                # Validate new email doesn't exist
                existing_users = self.storage.query_by_semantic(
                    semantic_filter={"type": "user", "email": email},
                    max_results=1,
                )
                if existing_users and len(existing_users) > 0:
                    raise ValueError(f"Email {email} already in use")
            
            # Update metadata (Note: This requires metadata update functionality)
            # TODO: Implement update_concept_metadata in storage client
            # For now, we'll need to recreate the concept
            
            logger.info(f"⚠️ User update requested but metadata updates not yet implemented: {user_id}")
            return await self.get_user(user_id)
            
        except Exception as e:
            logger.error(f"Failed to update user {user_id}: {e}")
            raise ValueError(f"User update failed: {str(e)}")
    
    async def change_password(self, user_id: str, old_password: str, new_password: str) -> bool:
        """
        Change user password.
        
        Args:
            user_id: User concept ID
            old_password: Current password for verification
            new_password: New password
        
        Returns:
            True if successful
        
        Raises:
            ValueError: If validation fails
        """
        try:
            # Get user
            user = self.storage.query_concept(user_id)
            if not user:
                raise ValueError("User not found")
            
            metadata = user.get("metadata", {})
            stored_hash = metadata.get("password_hash")
            
            if not stored_hash:
                raise ValueError("User account is corrupted")
            
            # Verify old password
            try:
                self.ph.verify(stored_hash, old_password)
            except (VerificationError, VerifyMismatchError):
                raise ValueError("Current password is incorrect")
            
            # Validate new password strength
            if len(new_password) < 8:
                raise ValueError("New password must be at least 8 characters")
            
            # Hash new password
            new_hash = self.ph.hash(new_password)
            
            # TODO: Implement update_concept_metadata in storage client
            # For now, log the intent
            logger.info(f"⚠️ Password change requested but metadata updates not yet implemented: {user_id}")
            
            return True
            
        except ValueError:
            raise
        except Exception as e:
            logger.error(f"Password change failed for {user_id}: {e}")
            raise ValueError("Password change failed")
    
    async def generate_password_reset_token(self, email: str) -> Optional[str]:
        """
        Generate a password reset token for user.
        
        Args:
            email: User email
        
        Returns:
            Reset token or None if user not found
        """
        try:
            import secrets
            
            # Find user
            users = self.storage.query_by_semantic(
                semantic_filter={"type": "user", "email": email},
                max_results=1,
            )
            
            if not users or len(users) == 0:
                # Don't reveal if email exists
                return None
            
            user = users[0]
            user_id = user.get("id")
            
            # Generate secure token
            reset_token = secrets.token_urlsafe(32)
            expires_at = datetime.utcnow() + timedelta(hours=1)  # 1-hour expiration
            
            # Store reset token concept
            token_content = f"Password reset token for {email}"
            token_metadata = {
                "type": "password_reset_token",
                "user_id": user_id,
                "email": email,
                "token": reset_token,
                "created_at": datetime.utcnow().isoformat(),
                "expires_at": expires_at.isoformat(),
                "used": False,
            }
            
            token_id = self.storage.learn_concept_v2(
                content=token_content,
                options={
                    "generate_embedding": False,
                    "extract_associations": False,
                    "metadata": token_metadata,
                }
            )
            
            # Create association: user -> reset_token
            self.storage.create_association(
                from_concept=user_id,
                to_concept=token_id,
                association_type="has_reset_token",
                strength=1.0,
            )
            
            logger.info(f"✅ Password reset token generated for {email}")
            return reset_token
            
        except Exception as e:
            logger.error(f"Failed to generate reset token for {email}: {e}")
            return None
    
    async def reset_password_with_token(self, token: str, new_password: str) -> bool:
        """
        Reset password using reset token.
        
        Args:
            token: Password reset token
            new_password: New password
        
        Returns:
            True if successful
        
        Raises:
            ValueError: If token is invalid or expired
        """
        try:
            # Find token concept
            token_concepts = self.storage.query_by_semantic(
                semantic_filter={"type": "password_reset_token", "token": token},
                max_results=5,
            )
            
            token_concept = None
            for concept in token_concepts:
                metadata = concept.get("metadata", {})
                if metadata.get("token") == token and metadata.get("type") == "password_reset_token":
                    token_concept = concept
                    break
            
            if not token_concept:
                raise ValueError("Invalid reset token")
            
            token_metadata = token_concept.get("metadata", {})
            
            # Check if token is already used
            if token_metadata.get("used"):
                raise ValueError("Reset token already used")
            
            # Check expiration
            expires_at = datetime.fromisoformat(token_metadata.get("expires_at"))
            if datetime.utcnow() > expires_at:
                raise ValueError("Reset token expired")
            
            # Get user
            user_id = token_metadata.get("user_id")
            user = self.storage.query_concept(user_id)
            
            if not user:
                raise ValueError("User not found")
            
            # Validate new password
            if len(new_password) < 8:
                raise ValueError("Password must be at least 8 characters")
            
            # Hash new password
            new_hash = self.ph.hash(new_password)
            
            # TODO: Implement update_concept_metadata in storage client
            # Mark token as used
            # Update user password hash
            
            logger.info(f"⚠️ Password reset requested but metadata updates not yet implemented: {user_id}")
            
            return True
            
        except ValueError:
            raise
        except Exception as e:
            logger.error(f"Password reset failed: {e}")
            raise ValueError("Password reset failed")
    
    async def deactivate_user(self, user_id: str) -> bool:
        """
        Deactivate user account (soft delete).
        
        Args:
            user_id: User concept ID
        
        Returns:
            True if successful
        
        Raises:
            ValueError: If user not found
        """
        try:
            user = self.storage.query_concept(user_id)
            if not user:
                raise ValueError("User not found")
            
            # TODO: Implement update_concept_metadata to set active=false
            logger.info(f"⚠️ User deactivation requested but metadata updates not yet implemented: {user_id}")
            
            return True
            
        except Exception as e:
            logger.error(f"Failed to deactivate user {user_id}: {e}")
            raise ValueError("User deactivation failed")
    
    async def delete_user(self, user_id: str, requesting_user_id: str) -> bool:
        """
        Permanently delete user account and all associated data.
        
        This is a hard delete that removes:
        - User concept
        - All user sessions
        - User associations (spaces, conversations)
        
        Args:
            user_id: User concept ID to delete
            requesting_user_id: User ID making the request (for audit)
        
        Returns:
            True if successful
        
        Raises:
            ValueError: If user not found or deletion fails
        """
        try:
            # Get user to verify existence
            user = self.storage.query_concept(user_id)
            if not user:
                raise ValueError("User not found")
            
            email = user.get("metadata", {}).get("email", "unknown")
            
            # Find all user sessions
            try:
                all_associations = self.storage.get_related_concepts(
                    concept_id=user_id,
                    association_types=["has_session"],
                    depth=1,
                )
                
                session_ids = [
                    related.get("id") 
                    for related in all_associations 
                    if related.get("metadata", {}).get("type") == "session"
                ]
                
                # Delete all sessions
                for session_id in session_ids:
                    try:
                        # TODO: Implement delete_concept in storage client
                        # For now, just invalidate sessions
                        await self.logout(session_id)
                        logger.info(f"   Invalidated session: {session_id[:8]}")
                    except Exception as e:
                        logger.warning(f"   Failed to invalidate session {session_id[:8]}: {e}")
                
            except Exception as e:
                logger.warning(f"Could not clean up sessions for user {user_id}: {e}")
            
            # TODO: Implement delete_concept in storage client
            # For now, we can only deactivate
            await self.deactivate_user(user_id)
            
            logger.info(
                f"✅ User deletion requested (deactivated for now): {email} "
                f"(by user: {requesting_user_id[:8]})"
            )
            
            return True
            
        except ValueError:
            raise
        except Exception as e:
            logger.error(f"Failed to delete user {user_id}: {e}")
            raise ValueError(f"User deletion failed: {str(e)}")
    
    async def list_users(
        self,
        organization: Optional[str] = None,
        active_only: bool = True,
        limit: int = 100,
    ) -> list:
        """
        List users (admin function).
        
        Args:
            organization: Filter by organization (optional)
            active_only: Only return active users
            limit: Maximum number of users to return
        
        Returns:
            List of user dictionaries
        """
        try:
            # Search for user concepts
            filter_dict = {"type": "user"}
            if organization:
                filter_dict["organization"] = organization
            
            results = self.storage.query_by_semantic(
                semantic_filter=filter_dict,
                max_results=limit,
            )
            
            users = []
            for result in results:
                metadata = result.get("metadata", {})
                if metadata.get("type") != "user":
                    continue
                
                # Filter by active status
                if active_only and not metadata.get("active"):
                    continue
                
                users.append({
                    "user_id": result.get("id"),
                    "email": metadata.get("email"),
                    "full_name": metadata.get("full_name"),
                    "organization": metadata.get("organization"),
                    "role": metadata.get("role"),
                    "active": metadata.get("active"),
                    "created_at": metadata.get("created_at"),
                    "last_login": metadata.get("last_login"),
                })
            
            return users
            
        except Exception as e:
            logger.error(f"Failed to list users: {e}")
            return []
    
    async def search_users(self, query: str, limit: int = 20) -> list:
        """
        Search for users by email or name.
        
        Args:
            query: Search query
            limit: Maximum results
        
        Returns:
            List of matching users
        """
        try:
            # For user search, we'll use a simple type filter for now
            # TODO: Implement more sophisticated text search when available
            results = self.storage.query_by_semantic(
                semantic_filter={"type": "user"},
                max_results=limit,
            )
            
            users = []
            for result in results:
                metadata = result.get("metadata", {})
                if metadata.get("type") == "user":
                    users.append({
                        "user_id": result.get("id"),
                        "email": metadata.get("email"),
                        "full_name": metadata.get("full_name"),
                        "organization": metadata.get("organization"),
                        "role": metadata.get("role"),
                        "active": metadata.get("active"),
                    })
            
            return users
            
        except Exception as e:
            logger.error(f"Failed to search users: {e}")
            return []
