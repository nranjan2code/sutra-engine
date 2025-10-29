/**
 * Sutra UI Components - Card
 * 
 * Versatile container component with theme-aware styling.
 */

import React, { forwardRef } from 'react';
import { useTheme, cn } from '@sutra/ui-core';

export interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Card variant */
  variant?: 'default' | 'elevated' | 'outlined' | 'floating';
  
  /** Padding size */
  padding?: 'none' | 'sm' | 'md' | 'lg';
  
  /** Whether card is interactive (clickable) */
  interactive?: boolean;
  
  /** Children content */
  children?: React.ReactNode;
}

export const Card = forwardRef<HTMLDivElement, CardProps>(
  (
    {
      variant = 'default',
      padding = 'md',
      interactive = false,
      className,
      children,
      ...props
    },
    ref
  ) => {
    const { theme } = useTheme();
    const cardTokens = theme.components.card;
    
    const paddingStyles: React.CSSProperties = {
      none: { padding: '0' },
      sm: { padding: theme.tokens.spacing.scale['3'] },
      md: { padding: theme.tokens.spacing.scale['4'] },
      lg: { padding: theme.tokens.spacing.scale['6'] },
    }[padding];
    
    const variantStyles: React.CSSProperties = {
      default: {
        background: cardTokens.background,
        border: cardTokens.border,
        boxShadow: cardTokens.boxShadow,
      },
      elevated: {
        background: cardTokens.background,
        border: 'none',
        boxShadow: theme.tokens.elevation.md,
      },
      outlined: {
        background: 'transparent',
        border: cardTokens.border,
        boxShadow: 'none',
      },
      floating: {
        background: cardTokens.background,
        border: cardTokens.border,
        boxShadow: theme.tokens.elevation.xl,
      },
    }[variant];
    
    const baseStyles: React.CSSProperties = {
      borderRadius: cardTokens.borderRadius,
      transition: `all ${theme.tokens.animation.duration.normal} ${theme.tokens.animation.easing.easeInOut}`,
      cursor: interactive ? 'pointer' : 'default',
      ...variantStyles,
      ...paddingStyles,
    };
    
    return (
      <div
        ref={ref}
        className={cn('sutra-card', `sutra-card--${variant}`, interactive && 'sutra-card--interactive', className)}
        style={baseStyles}
        {...props}
      >
        {children}
      </div>
    );
  }
);

Card.displayName = 'Card';

// Card sub-components for composition
export interface CardHeaderProps extends React.HTMLAttributes<HTMLDivElement> {
  children?: React.ReactNode;
}

export const CardHeader = forwardRef<HTMLDivElement, CardHeaderProps>(
  ({ className, children, ...props }, ref) => {
    const { theme } = useTheme();
    
    return (
      <div
        ref={ref}
        className={cn('sutra-card__header', className)}
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          marginBottom: theme.tokens.spacing.scale['4'],
        }}
        {...props}
      >
        {children}
      </div>
    );
  }
);

CardHeader.displayName = 'CardHeader';

export interface CardContentProps extends React.HTMLAttributes<HTMLDivElement> {
  children?: React.ReactNode;
}

export const CardContent = forwardRef<HTMLDivElement, CardContentProps>(
  ({ className, children, ...props }, ref) => {
    return (
      <div
        ref={ref}
        className={cn('sutra-card__content', className)}
        {...props}
      >
        {children}
      </div>
    );
  }
);

CardContent.displayName = 'CardContent';

export interface CardActionsProps extends React.HTMLAttributes<HTMLDivElement> {
  children?: React.ReactNode;
}

export const CardActions = forwardRef<HTMLDivElement, CardActionsProps>(
  ({ className, children, ...props }, ref) => {
    const { theme } = useTheme();
    
    return (
      <div
        ref={ref}
        className={cn('sutra-card__actions', className)}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: theme.tokens.spacing.scale['2'],
          marginTop: theme.tokens.spacing.scale['4'],
        }}
        {...props}
      >
        {children}
      </div>
    );
  }
);

CardActions.displayName = 'CardActions';

// Attach sub-components to Card
(Card as any).Header = CardHeader;
(Card as any).Content = CardContent;
(Card as any).Actions = CardActions;
