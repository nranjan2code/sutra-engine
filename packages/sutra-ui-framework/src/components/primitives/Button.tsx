/**
 * Sutra UI Components - Button
 * 
 * Primary interaction component with full theme support.
 * Performance-optimized with React.memo and useMemo.
 */

import React, { forwardRef, useMemo, memo } from 'react';
import { useTheme, cn, transition } from '../../core';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /** Button visual variant */
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
  
  /** Button size */
  size?: 'sm' | 'md' | 'lg';
  
  /** Loading state */
  loading?: boolean;
  
  /** Full width */
  fullWidth?: boolean;
  
  /** Icon before text */
  icon?: React.ReactNode;
  
  /** Icon after text */
  iconRight?: React.ReactNode;
  
  /** Button children */
  children?: React.ReactNode;
}

// Memoized Button component for performance optimization
const ButtonComponent = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = 'primary',
      size = 'md',
      loading = false,
      fullWidth = false,
      icon,
      iconRight,
      className,
      disabled,
      children,
      ...props
    },
    ref
  ) => {
    const { theme } = useTheme();
    const tokens = theme.components.button[variant];
    
    // Memoize style calculations to avoid recalculation on every render
    const baseStyles = useMemo<React.CSSProperties>(() => ({
      display: 'inline-flex',
      alignItems: 'center',
      justifyContent: 'center',
      gap: theme.tokens.spacing.scale['2'],
      fontFamily: theme.tokens.typography.fontFamily.base,
      fontWeight: theme.tokens.typography.fontWeight.medium,
      lineHeight: theme.tokens.typography.lineHeight.tight,
      borderRadius: theme.tokens.shape.borderRadius.md,
      cursor: disabled || loading ? 'not-allowed' : 'pointer',
      transition: transition(['all'], theme.tokens.animation.duration.normal, theme.tokens.animation.easing.easeInOut),
      opacity: disabled || loading ? 0.6 : 1,
      width: fullWidth ? '100%' : 'auto',
      
      // Token-based styles
      background: tokens.background,
      color: tokens.color,
      border: tokens.border || 'none',
      boxShadow: tokens.boxShadow,
    }), [theme, tokens, disabled, loading, fullWidth]);
    
    const sizeStyles = useMemo<React.CSSProperties>(() => ({
      sm: {
        height: '32px',
        padding: `0 ${theme.tokens.spacing.scale['3']}`,
        fontSize: theme.tokens.typography.fontSize.sm,
      },
      md: {
        height: '40px',
        padding: `0 ${theme.tokens.spacing.scale['4']}`,
        fontSize: theme.tokens.typography.fontSize.base,
      },
      lg: {
        height: '48px',
        padding: `0 ${theme.tokens.spacing.scale['6']}`,
        fontSize: theme.tokens.typography.fontSize.lg,
      },
    }[size]), [theme, size]);
    
    const hoverStyles = useMemo<React.CSSProperties>(() => ({
      background: tokens.backgroundHover,
      color: tokens.colorHover || tokens.color,
      border: tokens.borderHover || tokens.border || 'none',
      boxShadow: tokens.boxShadowHover || tokens.boxShadow,
    }), [tokens]);
    
    return (
      <button
        ref={ref}
        className={cn('sutra-button', `sutra-button--${variant}`, `sutra-button--${size}`, className)}
        disabled={disabled || loading}
        aria-busy={loading}
        style={{ ...baseStyles, ...sizeStyles }}
        onMouseEnter={(e) => {
          if (!disabled && !loading) {
            Object.assign(e.currentTarget.style, hoverStyles);
          }
        }}
        onMouseLeave={(e) => {
          Object.assign(e.currentTarget.style, baseStyles, sizeStyles);
        }}
        {...props}
      >
        {loading && <Spinner size={size === 'sm' ? 14 : size === 'lg' ? 20 : 16} />}
        {!loading && icon && <span className="sutra-button__icon">{icon}</span>}
        {children && <span className="sutra-button__text">{children}</span>}
        {!loading && iconRight && <span className="sutra-button__icon-right">{iconRight}</span>}
      </button>
    );
  }
);

// Export memoized component with display name
export const Button = memo(ButtonComponent);
Button.displayName = 'Button';

// Simple spinner component
function Spinner({ size }: { size: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      style={{
        animation: 'spin 1s linear infinite',
      }}
    >
      <circle
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="4"
        strokeOpacity="0.25"
      />
      <path
        d="M12 2a10 10 0 0 1 10 10"
        stroke="currentColor"
        strokeWidth="4"
        strokeLinecap="round"
      />
      <style>{`
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </svg>
  );
}
