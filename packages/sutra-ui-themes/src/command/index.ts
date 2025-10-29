/**
 * Sutra UI Themes - Command Theme
 * 
 * Dark command center theme for sutra-control.
 * Dashboard-focused with subtle glow effects.
 */

import { createTheme, type Theme } from '@sutra/ui-core';
import { baseTokens } from '../base/tokens';

/**
 * Command Theme - Dark Command Center
 * 
 * Design Philosophy:
 * - Dark blue-gray aesthetic for command centers
 * - Indigo/cyan accent colors
 * - Moderate contrast for extended viewing
 * - Subtle glow effects (less intense than holographic)
 * - WCAG AA+ compliant (12.0:1 contrast)
 * - Professional yet tech-forward
 */
export const commandTheme: Theme = createTheme({
  name: 'command',
  displayName: 'Command Center',
  description: 'Dark command center interface for system monitoring',

  tokens: {
    // Spacing - from baseTokens
    spacing: baseTokens.spacing!,
    // Shape - from baseTokens
    shape: baseTokens.shape!,
    // Animation - from baseTokens
    animation: baseTokens.animation!,
    // Breakpoints - from baseTokens
    breakpoints: baseTokens.breakpoints!,
    // Z-Index - from baseTokens
    zIndex: baseTokens.zIndex!,

    // Color System - Indigo/Cyan
    color: {
      primary: '#6366f1',           // Indigo
      secondary: '#06b6d4',         // Cyan
      tertiary: '#8b5cf6',          // Purple

      success: '#10b981',           // Green
      warning: '#f59e0b',           // Amber
      error: '#ef4444',             // Red
      info: '#3b82f6',              // Blue

      surface: '#1a2332',           // Dark blue-gray
      surfaceHover: '#222d40',      // Slightly lighter
      surfaceActive: '#2a384e',     // Even lighter
      surfaceVariant: '#1e2836',    // Subtle variant

      background: '#0f1629',        // Darker blue-gray
      backgroundSecondary: '#141c2e', // Dark blue-gray

      text: {
        primary: '#e3e8ef',         // Light gray
        secondary: '#c3c8d0',       // Mid gray
        tertiary: '#a3a8b0',        // Dark gray
        disabled: '#8d9199',        // Very dark gray
        inverse: '#0f1629',         // Dark (for light backgrounds)
      },

      border: {
        default: '#2d3748',         // Gray
        hover: '#6366f1',           // Indigo
        focus: '#6366f1',           // Indigo
        error: '#ef4444',           // Red
      },

      overlay: {
        background: 'rgba(15, 22, 41, 0.9)',
        backdrop: 'rgba(0, 0, 0, 0.6)',
      },
    },

    // Typography - System fonts
    typography: {
      ...baseTokens.typography!,
      fontFamily: {
        base: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
        heading: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
        mono: '"SF Mono", Monaco, "Cascadia Code", "Roboto Mono", monospace',
      },
    },

    // Elevation - Subtle glowing shadows
    elevation: {
      none: 'none',
      xs: '0 1px 2px rgba(0, 0, 0, 0.3)',
      sm: '0 2px 4px rgba(0, 0, 0, 0.3), 0 0 8px rgba(99, 102, 241, 0.1)',
      base: '0 4px 8px rgba(0, 0, 0, 0.4), 0 0 12px rgba(99, 102, 241, 0.15)',
      md: '0 8px 16px rgba(0, 0, 0, 0.4), 0 0 16px rgba(99, 102, 241, 0.2)',
      lg: '0 12px 24px rgba(0, 0, 0, 0.5), 0 0 24px rgba(99, 102, 241, 0.2)',
      xl: '0 16px 32px rgba(0, 0, 0, 0.6), 0 0 32px rgba(99, 102, 241, 0.25)',
      '2xl': '0 24px 48px rgba(0, 0, 0, 0.7), 0 0 48px rgba(99, 102, 241, 0.3)',
    },

    // Subtle effects
    effects: {
      glow: {
        enabled: true,
        color: '#6366f1',
        blur: [8, 16],
        opacity: [0.15, 0.1],
        spread: 1,
      },
      scanlines: {
        enabled: false,
      },
      frostedGlass: {
        enabled: true,
        blur: 10,
        opacity: 0.08,
        saturation: 1.1,
      },
      gradient: {
        primary: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)',
        secondary: 'linear-gradient(135deg, #06b6d4 0%, #3b82f6 100%)',
        accent: 'linear-gradient(135deg, #10b981 0%, #06b6d4 100%)',
      },
    },
  },

  // Component Tokens
  components: {
    // Button
    button: {
      primary: {
        background: '#6366f1',
        backgroundHover: '#7c3aed',
        backgroundActive: '#5b21b6',
        color: '#ffffff',
        border: 'none',
        boxShadow: '0 0 8px rgba(99, 102, 241, 0.3)',
        boxShadowHover: '0 0 16px rgba(99, 102, 241, 0.5)',
      },
      secondary: {
        background: 'transparent',
        backgroundHover: 'rgba(99, 102, 241, 0.1)',
        backgroundActive: 'rgba(99, 102, 241, 0.2)',
        color: '#6366f1',
        border: '1px solid #6366f1',
        boxShadow: '0 0 6px rgba(99, 102, 241, 0.2)',
      },
      ghost: {
        background: 'transparent',
        backgroundHover: 'rgba(255, 255, 255, 0.05)',
        backgroundActive: 'rgba(255, 255, 255, 0.1)',
        color: '#e3e8ef',
        border: 'none',
        boxShadow: 'none',
      },
      danger: {
        background: '#ef4444',
        backgroundHover: '#dc2626',
        backgroundActive: '#b91c1c',
        color: '#ffffff',
        border: 'none',
        boxShadow: '0 0 8px rgba(239, 68, 68, 0.3)',
      },
    },

    // Card
    card: {
      background: 'rgba(26, 35, 50, 0.9)',
      border: '1px solid rgba(99, 102, 241, 0.15)',
      borderRadius: '8px',
      boxShadow: '0 4px 8px rgba(0, 0, 0, 0.3), 0 0 12px rgba(99, 102, 241, 0.1)',
      padding: '1.25rem',
    },

    // Input
    input: {
      background: 'rgba(15, 22, 41, 0.6)',
      border: '1px solid rgba(99, 102, 241, 0.3)',
      borderFocus: '1px solid #6366f1',
      borderError: '1px solid #ef4444',
      borderRadius: '4px',
      padding: '0.625rem 0.875rem',
      color: '#e3e8ef',
      placeholder: '#a3a8b0',
    },
  },

  // Accessibility
  accessibility: {
    contrastRatio: 12.0, // WCAG AA+
    colorblindSafe: true,
    reducedMotion: true,
    focusVisible: {
      outlineColor: '#6366f1',
      outlineWidth: '2px',
      outlineOffset: '2px',
    },
  },

  // CSS Variables
  cssVariables: {
    '--command-glow': '0 0 16px rgba(99, 102, 241, 0.3)',
    '--command-blur': '10px',
  },
});

export default commandTheme;
