"""
Authentication routes.

Handles user registration, login, logout, and session management.

🔥 PRODUCTION SECURITY:
- httpOnly cookies for tokens (XSS immune)
- Secure flag for HTTPS-only
- SameSite=Lax for CSRF protection
"""

import logging
import os
from datetime import timedelta

from fastapi import APIRouter, Depends, HTTPException, Request, Response, status

from ..config import settings
from ..middleware.auth import (
    create_access_token,
    create_refresh_token,
    decode_token,
    get_current_user,
)
from ..models import (
    ChangePasswordRequest,
    ForgotPasswordRequest,
    LoginRequest,
    LoginResponse,
    LogoutResponse,
    PasswordResetResponse,
    RefreshTokenRequest,
    RegisterRequest,
    ResetPasswordRequest,
    UserResponse,
)
from ..services import UserService

logger = logging.getLogger(__name__)

# Create router
router = APIRouter(
    prefix="/auth",
    tags=["Authentication"],
)


def get_user_service(request: Request) -> UserService:
    """
    Dependency to get UserService instance.
    
    Args:
        request: FastAPI request
    
    Returns:
        UserService instance
    """
    if not hasattr(request.app.state, "user_storage_client"):
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail="User storage service not available"
        )
    
    return UserService(request.app.state.user_storage_client)


@router.post(
    "/register",
    response_model=UserResponse,
    status_code=status.HTTP_201_CREATED,
    summary="Register new user",
    description="Create a new user account with email and password."
)
async def register(
    request: RegisterRequest,
    user_service: UserService = Depends(get_user_service)
):
    """
    Register a new user account.
    
    Creates a User concept in user-storage.dat with securely hashed password.
    
    **Returns:**
    - User information (without password)
    
    **Raises:**
    - 400: Invalid input or user already exists
    - 500: Server error during registration
    """
    try:
        user_info = await user_service.register(
            email=request.email,
            password=request.password,
            organization=request.organization,
            full_name=request.full_name,
            role=request.role,
        )
        
        return UserResponse(**user_info)
        
    except ValueError as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=str(e)
        )
    except Exception as e:
        logger.error(f"Registration failed: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Registration failed"
        )


@router.post(
    "/login",
    response_model=LoginResponse,
    summary="User login",
    description="Authenticate user and create session with httpOnly cookies (production-grade security)."
)
async def login(
    response: Response,  # 🔥 PRODUCTION: FastAPI Response to set cookies
    request: LoginRequest,
    user_service: UserService = Depends(get_user_service)
):
    """
    Authenticate user and create session with httpOnly cookies.
    
    🔥 PRODUCTION SECURITY:
    - Tokens stored in httpOnly cookies (immune to XSS)
    - Secure flag for HTTPS-only transmission
    - SameSite=Lax for CSRF protection
    - No tokens in response body (client-side never sees them)
    
    **Returns:**
    - User information only (no tokens)
    
    **Raises:**
    - 401: Invalid credentials
    - 500: Server error during login
    """
    try:
        session_info = await user_service.login(
            email=request.email,
            password=request.password
        )
        
        # Create JWT tokens
        token_data = {
            "user_id": session_info["user_id"],
            "session_id": session_info["session_id"],
            "email": session_info["email"],
            "organization": session_info["organization"],
            "role": session_info["role"],
        }
        
        access_token = create_access_token(token_data)
        refresh_token = create_refresh_token(token_data)
        
        # 🔥 PRODUCTION: Set httpOnly cookies instead of returning tokens
        secure_mode = os.getenv("SUTRA_SECURE_MODE", "false").lower() == "true"
        
        # Access token cookie (24 hours)
        response.set_cookie(
            key="access_token",
            value=access_token,
            httponly=True,  # XSS protection
            secure=secure_mode,  # HTTPS only in production
            samesite="lax",  # CSRF protection
            max_age=settings.jwt_expiration_hours * 3600,
            path="/",
        )
        
        # Refresh token cookie (7 days)
        response.set_cookie(
            key="refresh_token",
            value=refresh_token,
            httponly=True,
            secure=secure_mode,
            samesite="lax",
            max_age=settings.jwt_refresh_expiration_days * 86400,
            path="/",
        )
        
        user_response = UserResponse(
            user_id=session_info["user_id"],
            email=session_info["email"],
            organization=session_info["organization"],
            role=session_info["role"],
            full_name=session_info.get("full_name"),
        )
        
        # 🔥 PRODUCTION: Return user info only (NO tokens in response body)
        return LoginResponse(
            access_token="<set-in-cookie>",  # Not used by client
            refresh_token="<set-in-cookie>",  # Not used by client
            token_type="bearer",
            expires_in=settings.jwt_expiration_hours * 3600,
            user=user_response
        )
        
    except ValueError as e:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=str(e),
            headers={"WWW-Authenticate": "Bearer"},
        )
    except Exception as e:
        logger.error(f"Login failed: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Login failed"
        )


@router.post(
    "/logout",
    response_model=LogoutResponse,
    summary="User logout",
    description="Invalidate current session and clear httpOnly cookies."
)
async def logout(
    response: Response,  # 🔥 PRODUCTION: Clear cookies
    current_user: dict = Depends(get_current_user),
    user_service: UserService = Depends(get_user_service)
):
    """
    Logout user and invalidate session.
    
    🔥 PRODUCTION: Clears httpOnly cookies server-side.
    
    **Returns:**
    - Logout confirmation
    
    **Raises:**
    - 401: Invalid or expired token
    - 500: Server error during logout
    """
    try:
        session_id = current_user["session_id"]
        await user_service.logout(session_id)
        
        # 🔥 PRODUCTION: Clear httpOnly cookies
        response.delete_cookie(key="access_token", path="/")
        response.delete_cookie(key="refresh_token", path="/")
        
        return LogoutResponse(
            message="Successfully logged out",
            success=True
        )
        
    except Exception as e:
        logger.error(f"Logout failed: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Logout failed"
        )


@router.get(
    "/me",
    response_model=UserResponse,
    summary="Get current user",
    description="Get information about the currently authenticated user."
)
async def get_current_user_info(
    current_user: dict = Depends(get_current_user),
    user_service: UserService = Depends(get_user_service)
):
    """
    Get current user information.
    
    Validates session and returns user details from user-storage.dat.
    
    **Returns:**
    - User information
    
    **Raises:**
    - 401: Invalid or expired token
    - 404: User not found
    """
    try:
        # Validate session
        session_info = await user_service.validate_session(
            current_user["session_id"]
        )
        
        if not session_info:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Session expired or invalid",
                headers={"WWW-Authenticate": "Bearer"},
            )
        
        # Get full user details
        user_info = await user_service.get_user(session_info["user_id"])
        
        if not user_info:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail="User not found"
            )
        
        return UserResponse(**user_info)
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Failed to get user info: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Failed to retrieve user information"
        )


@router.post(
    "/refresh",
    response_model=LoginResponse,
    summary="Refresh access token",
    description="Use refresh token from httpOnly cookie to get new tokens."
)
async def refresh_token(
    request: Request,  # 🔥 PRODUCTION: Read cookie
    response: Response,  # 🔥 PRODUCTION: Set cookies
    user_service: UserService = Depends(get_user_service)
):
    """
    Refresh access token using httpOnly cookie.
    
    🔥 PRODUCTION: Reads refresh_token from httpOnly cookie,
    validates it, and sets new httpOnly cookies.
    
    **Returns:**
    - User information (tokens in httpOnly cookies)
    
    **Raises:**
    - 401: Invalid or expired refresh token
    - 500: Server error during refresh
    """
    try:
        # 🔥 PRODUCTION: Read refresh token from httpOnly cookie
        refresh_token_value = request.cookies.get("refresh_token")
        
        if not refresh_token_value:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Refresh token not found",
                headers={"WWW-Authenticate": "Bearer"},
            )
        
        # Decode refresh token
        payload = decode_token(refresh_token_value)
        
        # Validate session still exists
        session_id = payload.get("session_id")
        if not session_id:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Invalid refresh token",
                headers={"WWW-Authenticate": "Bearer"},
            )
        
        session_info = await user_service.validate_session(session_id)
        
        if not session_info:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="Session expired or invalid",
                headers={"WWW-Authenticate": "Bearer"},
            )
        
        # Create new tokens
        token_data = {
            "user_id": session_info["user_id"],
            "session_id": session_info["session_id"],
            "email": session_info["email"],
            "organization": session_info["organization"],
            "role": session_info["role"],
        }
        
        access_token = create_access_token(token_data)
        refresh_token_new = create_refresh_token(token_data)
        
        # 🔥 PRODUCTION: Set new httpOnly cookies
        secure_mode = os.getenv("SUTRA_SECURE_MODE", "false").lower() == "true"
        
        response.set_cookie(
            key="access_token",
            value=access_token,
            httponly=True,
            secure=secure_mode,
            samesite="lax",
            max_age=settings.jwt_expiration_hours * 3600,
            path="/",
        )
        
        response.set_cookie(
            key="refresh_token",
            value=refresh_token_new,
            httponly=True,
            secure=secure_mode,
            samesite="lax",
            max_age=settings.jwt_refresh_expiration_days * 86400,
            path="/",
        )
        
        user_response = UserResponse(
            user_id=session_info["user_id"],
            email=session_info["email"],
            organization=session_info["organization"],
            role=session_info["role"],
            full_name=session_info.get("full_name"),
        )
        
        # 🔥 PRODUCTION: Return user info only (tokens in cookies)
        return LoginResponse(
            access_token="<set-in-cookie>",
            refresh_token="<set-in-cookie>",
            token_type="bearer",
            expires_in=settings.jwt_expiration_hours * 3600,
            user=user_response
        )
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Token refresh failed: {e}")
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Token refresh failed",
            headers={"WWW-Authenticate": "Bearer"},
        )


@router.get(
    "/health",
    summary="Auth service health",
    description="Check authentication service health."
)
async def auth_health(request: Request):
    """
    Check authentication service health.
    
    Verifies user storage connection.
    
    **Returns:**
    - Health status
    """
    try:
        if not hasattr(request.app.state, "user_storage_client"):
            return {
                "status": "unhealthy",
                "message": "User storage not connected"
            }
        
        # Try to get stats
        try:
            stats = request.app.state.user_storage_client.stats()
            user_concepts = stats.get("concepts", 0)
        except:
            user_concepts = 0
        
        return {
            "status": "healthy",
            "message": "Authentication service operational",
            "user_concepts": user_concepts
        }
        
    except Exception as e:
        logger.error(f"Auth health check failed: {e}")
        return {
            "status": "unhealthy",
            "message": str(e)
        }


@router.put(
    "/change-password",
    summary="Change password",
    description="Change user password (requires current password)."
)
async def change_password(
    request: ChangePasswordRequest,
    current_user: dict = Depends(get_current_user),
    user_service: UserService = Depends(get_user_service)
):
    """
    Change user password.
    
    Requires current password for verification.
    
    **Returns:**
    - Success confirmation
    
    **Raises:**
    - 400: Current password incorrect or validation failed
    - 401: Invalid or expired token
    - 500: Server error
    """
    try:
        success = await user_service.change_password(
            user_id=current_user["user_id"],
            old_password=request.old_password,
            new_password=request.new_password
        )
        
        if not success:
            raise HTTPException(
                status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
                detail="Password change failed"
            )
        
        return {
            "message": "Password changed successfully",
            "success": True
        }
        
    except ValueError as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=str(e)
        )
    except Exception as e:
        logger.error(f"Password change failed: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Password change failed"
        )


@router.post(
    "/forgot-password",
    response_model=PasswordResetResponse,
    summary="Request password reset",
    description="Request a password reset token (sent via email in production)."
)
async def forgot_password(
    request: ForgotPasswordRequest,
    user_service: UserService = Depends(get_user_service)
):
    """
    Request password reset token.
    
    Generates a secure token and returns it (in production, would send via email).
    Always returns success even if email doesn't exist (security best practice).
    
    **Returns:**
    - Success message
    
    **Note:** In production, the token would be sent via email, not returned in response.
    """
    try:
        token = await user_service.generate_password_reset_token(request.email)
        
        # In production, send token via email
        # For now, we return it (NOT secure for production!)
        if token:
            logger.info(f"Password reset token generated: {token[:8]}...")
        
        # Always return success (don't reveal if email exists)
        return PasswordResetResponse(
            message="If the email exists, a password reset link has been sent",
            success=True
        )
        
    except Exception as e:
        logger.error(f"Forgot password request failed: {e}")
        # Still return success (don't reveal errors)
        return PasswordResetResponse(
            message="If the email exists, a password reset link has been sent",
            success=True
        )


@router.post(
    "/reset-password",
    response_model=PasswordResetResponse,
    summary="Reset password with token",
    description="Reset password using the reset token."
)
async def reset_password(
    request: ResetPasswordRequest,
    user_service: UserService = Depends(get_user_service)
):
    """
    Reset password using token.
    
    Uses the token from forgot-password endpoint to set a new password.
    
    **Returns:**
    - Success confirmation
    
    **Raises:**
    - 400: Invalid or expired token
    - 500: Server error
    """
    try:
        success = await user_service.reset_password_with_token(
            token=request.token,
            new_password=request.new_password
        )
        
        if not success:
            raise HTTPException(
                status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
                detail="Password reset failed"
            )
        
        return PasswordResetResponse(
            message="Password reset successfully",
            success=True
        )
        
    except ValueError as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=str(e)
        )
    except Exception as e:
        logger.error(f"Password reset failed: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail="Password reset failed"
        )

