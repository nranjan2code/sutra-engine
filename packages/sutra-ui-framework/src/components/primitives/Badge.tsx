/**
 * Sutra UI Components - Badge
 * 
 * Status indicator and label component.
 */

import React, { forwardRef } from 'react';
import { useTheme, cn } from '../../core';

export interface BadgeProps extends React.HTMLAttributes<HTMLSpanElement> {
  /** Badge color scheme */
  colorScheme?: 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info' | 'neutral';
  
  /** Badge size */
  size?: 'sm' | 'md' | 'lg';
  
  /** Badge variant */
  variant?: 'solid' | 'outline' | 'subtle';
  
  /** Numeric value to display */
  value?: number | string;
  
  /** Children content */
  children?: React.ReactNode;
}

export const Badge = forwardRef<HTMLSpanElement, BadgeProps>(
  (
    {
      colorScheme = 'neutral',
      size = 'md',
      variant = 'solid',
      value,
      className,
      children,
      ...props
    },
    ref
  ) => {
    const { theme } = useTheme();
    
    const colorMap = {
      primary: theme.tokens.color.primary,
      secondary: theme.tokens.color.secondary,
      success: theme.tokens.color.success,
      warning: theme.tokens.color.warning,
      error: theme.tokens.color.error,
      info: theme.tokens.color.info,
      neutral: theme.tokens.color.text.secondary,
    };
    
    const color = colorMap[colorScheme];
    
    const sizeStyles: React.CSSProperties = {
      sm: {
        height: '18px',
        padding: `0 ${theme.tokens.spacing.scale['2']}`,
        fontSize: theme.tokens.typography.fontSize.xs,
      },
      md: {
        height: '22px',
        padding: `0 ${theme.tokens.spacing.scale['2']}`,
        fontSize: theme.tokens.typography.fontSize.sm,
      },
      lg: {
        height: '26px',
        padding: `0 ${theme.tokens.spacing.scale['3']}`,
        fontSize: theme.tokens.typography.fontSize.base,
      },
    }[size];
    
    const variantStyles: React.CSSProperties = {
      solid: {
        background: color,
        color: '#ffffff',
        border: 'none',
      },
      outline: {
        background: 'transparent',
        color: color,
        border: `1px solid ${color}`,
      },
      subtle: {
        background: `${color}22`, // 22 = ~13% opacity in hex
        color: color,
        border: 'none',
      },
    }[variant];
    
    const baseStyles: React.CSSProperties = {
      display: 'inline-flex',
      alignItems: 'center',
      justifyContent: 'center',
      borderRadius: theme.tokens.shape.borderRadius.full,
      fontFamily: theme.tokens.typography.fontFamily.base,
      fontWeight: theme.tokens.typography.fontWeight.medium,
      lineHeight: 1,
      whiteSpace: 'nowrap',
      ...sizeStyles,
      ...variantStyles,
    };
    
    const content = value !== undefined ? value : children;
    
    return (
      <span
        ref={ref}
        className={cn('sutra-badge', `sutra-badge--${colorScheme}`, `sutra-badge--${size}`, `sutra-badge--${variant}`, className)}
        style={baseStyles}
        {...props}
      >
        {content}
      </span>
    );
  }
);

Badge.displayName = 'Badge';
