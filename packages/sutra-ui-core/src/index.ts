/**
 * Sutra UI Core - Main Entry Point
 * 
 * Re-exports all public APIs from the core package.
 */

// Theme System
export { ThemeProvider, useTheme } from './theme/ThemeProvider';
export { createTheme, extendTheme } from './theme/createTheme';
export type {
  Theme,
  ThemeTokens,
  ThemeContextValue,
  CreateThemeOptions,
  ColorTokens,
  TypographyTokens,
  SpacingTokens,
  ShapeTokens,
  ElevationTokens,
  AnimationTokens,
  BreakpointTokens,
  ZIndexTokens,
  EffectTokens,
  ComponentTokens,
  AccessibilityConfig,
} from './theme/types';

// Hooks
export {
  useMediaQuery,
  useBreakpoint,
  useReducedMotion,
  useFocusVisible,
  useColorScheme,
  useThemeColor,
  useResponsiveValue,
  useViewportSize,
  useContrastText,
} from './hooks';
export type { Breakpoint, ColorScheme, ViewportSize } from './hooks';

// Utilities
export {
  cn,
  bem,
  hexToRgba,
  lighten,
  darken,
  contrastRatio,
  isAccessible,
  transition,
  spacing,
  clamp,
  pxToRem,
  remToPx,
  srOnly,
  focusVisible,
  isHexColor,
  isCSSLength,
} from './utils';
