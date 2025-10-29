/**
 * Sutra UI Components - Text
 * 
 * Typography component with semantic variants.
 */

import React, { forwardRef } from 'react';
import { useTheme, cn } from '@sutra/ui-core';

export interface TextProps extends React.HTMLAttributes<HTMLElement> {
  /** Text semantic variant */
  variant?: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6' | 'body1' | 'body2' | 'caption' | 'overline';
  
  /** Text color */
  color?: 'primary' | 'secondary' | 'tertiary' | 'disabled' | 'inherit';
  
  /** Font weight */
  weight?: 'light' | 'normal' | 'medium' | 'semibold' | 'bold';
  
  /** Text alignment */
  align?: 'left' | 'center' | 'right';
  
  /** HTML element to render */
  as?: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6' | 'p' | 'span' | 'div';
  
  /** Children content */
  children?: React.ReactNode;
}

export const Text = forwardRef<HTMLElement, TextProps>(
  (
    {
      variant = 'body1',
      color = 'primary',
      weight,
      align = 'left',
      as,
      className,
      children,
      ...props
    },
    ref
  ) => {
    const { theme } = useTheme();
    
    // Determine element type
    const elementMap = {
      h1: 'h1',
      h2: 'h2',
      h3: 'h3',
      h4: 'h4',
      h5: 'h5',
      h6: 'h6',
      body1: 'p',
      body2: 'p',
      caption: 'span',
      overline: 'span',
    };
    
    const Element = (as || elementMap[variant]) as any;
    
    // Variant styles
    const variantStyles: React.CSSProperties = {
      h1: {
        fontSize: theme.tokens.typography.fontSize['4xl'],
        fontWeight: theme.tokens.typography.fontWeight.bold,
        lineHeight: theme.tokens.typography.lineHeight.tight,
      },
      h2: {
        fontSize: theme.tokens.typography.fontSize['3xl'],
        fontWeight: theme.tokens.typography.fontWeight.bold,
        lineHeight: theme.tokens.typography.lineHeight.tight,
      },
      h3: {
        fontSize: theme.tokens.typography.fontSize['2xl'],
        fontWeight: theme.tokens.typography.fontWeight.semibold,
        lineHeight: theme.tokens.typography.lineHeight.tight,
      },
      h4: {
        fontSize: theme.tokens.typography.fontSize.xl,
        fontWeight: theme.tokens.typography.fontWeight.semibold,
        lineHeight: theme.tokens.typography.lineHeight.normal,
      },
      h5: {
        fontSize: theme.tokens.typography.fontSize.lg,
        fontWeight: theme.tokens.typography.fontWeight.medium,
        lineHeight: theme.tokens.typography.lineHeight.normal,
      },
      h6: {
        fontSize: theme.tokens.typography.fontSize.base,
        fontWeight: theme.tokens.typography.fontWeight.medium,
        lineHeight: theme.tokens.typography.lineHeight.normal,
      },
      body1: {
        fontSize: theme.tokens.typography.fontSize.base,
        fontWeight: theme.tokens.typography.fontWeight.normal,
        lineHeight: theme.tokens.typography.lineHeight.normal,
      },
      body2: {
        fontSize: theme.tokens.typography.fontSize.sm,
        fontWeight: theme.tokens.typography.fontWeight.normal,
        lineHeight: theme.tokens.typography.lineHeight.normal,
      },
      caption: {
        fontSize: theme.tokens.typography.fontSize.xs,
        fontWeight: theme.tokens.typography.fontWeight.normal,
        lineHeight: theme.tokens.typography.lineHeight.normal,
      },
      overline: {
        fontSize: theme.tokens.typography.fontSize.xs,
        fontWeight: theme.tokens.typography.fontWeight.medium,
        lineHeight: theme.tokens.typography.lineHeight.normal,
        textTransform: 'uppercase' as const,
        letterSpacing: '0.05em',
      },
    }[variant];
    
    // Color styles
    const colorMap = {
      primary: theme.tokens.color.text.primary,
      secondary: theme.tokens.color.text.secondary,
      tertiary: theme.tokens.color.text.tertiary,
      disabled: theme.tokens.color.text.disabled,
      inherit: 'inherit',
    };
    
    const baseStyles: React.CSSProperties = {
      fontFamily: theme.tokens.typography.fontFamily.base,
      color: colorMap[color],
      textAlign: align,
      margin: 0,
      ...variantStyles,
    };
    
    // Override font weight if specified
    if (weight) {
      baseStyles.fontWeight = theme.tokens.typography.fontWeight[weight];
    }
    
    return (
      <Element
        ref={ref}
        className={cn('sutra-text', `sutra-text--${variant}`, className)}
        style={baseStyles}
        {...props}
      >
        {children}
      </Element>
    );
  }
);

Text.displayName = 'Text';
