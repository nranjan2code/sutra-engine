/**
 * Sutra UI Components - Main Entry Point
 * 
 * Exports all component primitives.
 */

// Primitives
export { Button } from './primitives/Button';
export type { ButtonProps } from './primitives/Button';

export { Card, CardHeader, CardContent, CardActions } from './primitives/Card';
export type { CardProps, CardHeaderProps, CardContentProps, CardActionsProps } from './primitives/Card';

export { Badge } from './primitives/Badge';
export type { BadgeProps } from './primitives/Badge';

export { Text } from './primitives/Text';
export type { TextProps } from './primitives/Text';

export { Input } from './primitives/Input';
export type { InputProps } from './primitives/Input';

// Re-export core utilities for convenience
export { cn, useTheme } from '@sutra/ui-core';
