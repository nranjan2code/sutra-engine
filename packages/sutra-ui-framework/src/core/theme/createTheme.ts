/**
 * Sutra UI Core - Theme Factory
 * 
 * Utility functions for creating and customizing themes.
 * Provides deep merging and validation for theme tokens.
 */

import type { Theme, CreateThemeOptions, ThemeTokens, ComponentTokens } from './types';

// ============================================================================
// Theme Factory
// ============================================================================

/**
 * Create a new theme or extend an existing theme
 * 
 * @example
 * ```typescript
 * const myTheme = createTheme({
 *   name: 'my-theme',
 *   displayName: 'My Theme',
 *   tokens: {
 *     color: {
 *       primary: '#ff0000',
 *     },
 *   },
 * });
 * ```
 */
export function createTheme(config: Partial<Theme> & Pick<Theme, 'name' | 'displayName'>): Theme {
  const defaultTokens = getDefaultTokens();
  const defaultComponents = getDefaultComponents();
  const defaultAccessibility = getDefaultAccessibility();
  
  return {
    name: config.name,
    displayName: config.displayName,
    description: config.description,
    tokens: deepMerge(defaultTokens, config.tokens || {}) as ThemeTokens,
    components: deepMerge(defaultComponents, config.components || {}) as ComponentTokens,
    accessibility: { ...defaultAccessibility, ...config.accessibility },
    cssVariables: config.cssVariables,
  };
}

/**
 * Extend an existing theme with overrides
 * 
 * @example
 * ```typescript
 * const darkTheme = extendTheme(baseTheme, {
 *   tokens: {
 *     color: {
 *       background: '#000000',
 *     },
 *   },
 * });
 * ```
 */
export function extendTheme(baseTheme: Theme, options: CreateThemeOptions): Theme {
  return {
    ...baseTheme,
    tokens: deepMerge(baseTheme.tokens, options.tokens || {}) as ThemeTokens,
    components: deepMerge(baseTheme.components, options.components || {}) as ComponentTokens,
    accessibility: { ...baseTheme.accessibility, ...options.accessibility },
  };
}

// ============================================================================
// Default Tokens
// ============================================================================

function getDefaultTokens(): ThemeTokens {
  return {
    color: {
      primary: '#6366f1',
      secondary: '#8b5cf6',
      success: '#10b981',
      warning: '#f59e0b',
      error: '#ef4444',
      info: '#3b82f6',
      surface: '#ffffff',
      background: '#f9fafb',
      text: {
        primary: '#111827',
        secondary: '#6b7280',
        disabled: '#9ca3af',
      },
      border: {
        default: '#e5e7eb',
      },
    },
    typography: {
      fontFamily: {
        base: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
      },
      fontSize: {
        xs: '0.75rem',
        sm: '0.875rem',
        base: '1rem',
        lg: '1.125rem',
        xl: '1.25rem',
        '2xl': '1.5rem',
        '3xl': '1.875rem',
        '4xl': '2.25rem',
      },
      fontWeight: {
        light: 300,
        normal: 400,
        medium: 500,
        semibold: 600,
        bold: 700,
      },
      lineHeight: {
        tight: 1.25,
        normal: 1.5,
        relaxed: 1.75,
        loose: 2,
      },
    },
    spacing: {
      base: 4,
      scale: {
        '0': '0',
        '1': '0.25rem',
        '2': '0.5rem',
        '3': '0.75rem',
        '4': '1rem',
        '5': '1.25rem',
        '6': '1.5rem',
        '8': '2rem',
        '10': '2.5rem',
        '12': '3rem',
        '16': '4rem',
        '20': '5rem',
        '24': '6rem',
        '32': '8rem',
      },
    },
    shape: {
      borderRadius: {
        none: '0',
        sm: '0.125rem',
        base: '0.25rem',
        md: '0.375rem',
        lg: '0.5rem',
        xl: '0.75rem',
        '2xl': '1rem',
        full: '9999px',
      },
      borderWidth: {
        none: '0',
        thin: '1px',
        base: '1px',
        thick: '2px',
      },
    },
    elevation: {
      none: 'none',
      xs: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
      sm: '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)',
      base: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
      md: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
      lg: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
      xl: '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
      '2xl': '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
    },
    animation: {
      duration: {
        instant: '0ms',
        fast: '150ms',
        normal: '250ms',
        slow: '350ms',
        slower: '500ms',
      },
      easing: {
        linear: 'linear',
        easeIn: 'cubic-bezier(0.4, 0, 1, 1)',
        easeOut: 'cubic-bezier(0, 0, 0.2, 1)',
        easeInOut: 'cubic-bezier(0.4, 0, 0.2, 1)',
        spring: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
      },
    },
    breakpoints: {
      xs: '0px',
      sm: '640px',
      md: '768px',
      lg: '1024px',
      xl: '1280px',
      '2xl': '1536px',
    },
    zIndex: {
      base: 0,
      dropdown: 1000,
      sticky: 1100,
      fixed: 1200,
      modalBackdrop: 1300,
      modal: 1400,
      popover: 1500,
      tooltip: 1600,
    },
  };
}

// ============================================================================
// Default Component Tokens
// ============================================================================

function getDefaultComponents(): ComponentTokens {
  return {
    button: {
      primary: {
        background: 'var(--sutra-color-primary)',
        backgroundHover: 'var(--sutra-color-primary)',
        color: '#ffffff',
        border: 'none',
      },
      secondary: {
        background: 'transparent',
        backgroundHover: 'var(--sutra-color-surface)',
        color: 'var(--sutra-color-primary)',
        border: '1px solid var(--sutra-color-border)',
      },
      ghost: {
        background: 'transparent',
        backgroundHover: 'var(--sutra-color-surface)',
        color: 'var(--sutra-color-text-primary)',
        border: 'none',
      },
      danger: {
        background: 'var(--sutra-color-error)',
        backgroundHover: 'var(--sutra-color-error)',
        color: '#ffffff',
        border: 'none',
      },
    },
    card: {
      background: 'var(--sutra-color-surface)',
      border: '1px solid var(--sutra-color-border)',
      borderRadius: 'var(--sutra-border-radius-base)',
      boxShadow: 'var(--sutra-elevation-sm)',
      padding: 'var(--sutra-spacing-4)',
    },
    input: {
      background: 'var(--sutra-color-surface)',
      border: '1px solid var(--sutra-color-border)',
      borderFocus: '1px solid var(--sutra-color-primary)',
      borderError: '1px solid var(--sutra-color-error)',
      borderRadius: 'var(--sutra-border-radius-base)',
      padding: 'var(--sutra-spacing-2) var(--sutra-spacing-3)',
      color: 'var(--sutra-color-text-primary)',
      placeholder: 'var(--sutra-color-text-secondary)',
    },
  };
}

// ============================================================================
// Default Accessibility Configuration
// ============================================================================

function getDefaultAccessibility() {
  return {
    contrastRatio: 4.5, // WCAG AA
    colorblindSafe: true,
    reducedMotion: true,
    focusVisible: {
      outlineColor: 'var(--sutra-color-primary)',
      outlineWidth: '2px',
      outlineOffset: '2px',
    },
  };
}

// ============================================================================
// Utilities
// ============================================================================

/**
 * Deep merge two objects
 */
function deepMerge<T extends Record<string, any>>(target: T, source: Partial<T>): T {
  const result = { ...target };
  
  for (const key in source) {
    if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
      result[key] = deepMerge(
        (result[key] || {}) as T[Extract<keyof T, string>], 
        source[key] as any
      );
    } else if (source[key] !== undefined) {
      result[key] = source[key] as any;
    }
  }
  
  return result;
}
