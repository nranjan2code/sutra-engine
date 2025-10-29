/**
 * Sutra UI Themes - Professional Theme
 * 
 * Material Design 3 inspired theme for sutra-client.
 * Clean, accessible, and professional for business users.
 */

import { createTheme, type Theme } from '@sutra/ui-core';
import { baseTokens } from '../base/tokens';

/**
 * Professional Theme - Material Design 3
 * 
 * Design Philosophy:
 * - Clean, modern Material Design 3 aesthetic
 * - Purple primary color scheme
 * - Light background for readability
 * - Subtle shadows and elevation
 * - WCAG AA compliant (7.0:1 contrast)
 * - Professional and trustworthy
 */
export const professionalTheme: Theme = createTheme({
  name: 'professional',
  displayName: 'Professional',
  description: 'Clean, modern interface for business users',

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

    // Color System - Purple Primary (Material Design 3)
    color: {
      primary: '#6750A4',           // Material purple
      secondary: '#625B71',         // Gray-purple
      tertiary: '#7D5260',          // Rose-purple

      success: '#198754',           // Green
      warning: '#FFC107',           // Amber
      error: '#DC3545',             // Red
      info: '#0DCAF0',              // Cyan

      surface: '#FFFFFF',           // White
      surfaceHover: '#F8F9FA',      // Very light gray
      surfaceActive: '#E9ECEF',     // Light gray
      surfaceVariant: '#E7E0EC',    // Light purple tint

      background: '#FEF7FF',        // Very light purple tint
      backgroundSecondary: '#FFFFFF', // White

      text: {
        primary: '#1C1B1F',         // Near black
        secondary: '#49454F',       // Dark gray
        tertiary: '#79747E',        // Mid gray
        disabled: '#9CA3AF',        // Light gray
        inverse: '#FFFFFF',         // White (for dark backgrounds)
      },

      border: {
        default: '#CAC4D0',         // Light gray-purple
        hover: '#6750A4',           // Primary purple
        focus: '#6750A4',           // Primary purple
        error: '#DC3545',           // Red
      },

      overlay: {
        background: 'rgba(0, 0, 0, 0.5)',
        backdrop: 'rgba(0, 0, 0, 0.3)',
      },
    },

    // Typography - System sans-serif
    typography: {
      ...baseTokens.typography!,
      fontFamily: {
        base: '"Roboto", -apple-system, BlinkMacSystemFont, "Segoe UI", "Helvetica Neue", Arial, sans-serif',
        heading: '"Roboto", -apple-system, BlinkMacSystemFont, "Segoe UI", "Helvetica Neue", Arial, sans-serif',
        mono: '"Roboto Mono", "SF Mono", Monaco, monospace',
      },
    },

    // Elevation - Subtle shadows
    elevation: {
      none: 'none',
      xs: '0 1px 2px rgba(0, 0, 0, 0.05)',
      sm: '0 1px 3px rgba(0, 0, 0, 0.1), 0 1px 2px rgba(0, 0, 0, 0.06)',
      base: '0 4px 6px rgba(0, 0, 0, 0.1), 0 2px 4px rgba(0, 0, 0, 0.06)',
      md: '0 10px 15px rgba(0, 0, 0, 0.1), 0 4px 6px rgba(0, 0, 0, 0.05)',
      lg: '0 20px 25px rgba(0, 0, 0, 0.1), 0 10px 10px rgba(0, 0, 0, 0.04)',
      xl: '0 25px 50px rgba(0, 0, 0, 0.15)',
      '2xl': '0 25px 50px rgba(0, 0, 0, 0.25)',
    },

    // No special effects for professional theme
    effects: {
      glow: {
        enabled: false,
      },
      scanlines: {
        enabled: false,
      },
      frostedGlass: {
        enabled: false,
      },
    },
  },

  // Component Tokens
  components: {
    // Button
    button: {
      primary: {
        background: '#6750A4',
        backgroundHover: '#7965AF',
        backgroundActive: '#5A46A0',
        color: '#FFFFFF',
        border: 'none',
        boxShadow: '0 2px 4px rgba(103, 80, 164, 0.2)',
        boxShadowHover: '0 4px 8px rgba(103, 80, 164, 0.3)',
      },
      secondary: {
        background: 'transparent',
        backgroundHover: 'rgba(103, 80, 164, 0.08)',
        backgroundActive: 'rgba(103, 80, 164, 0.12)',
        color: '#6750A4',
        border: '1px solid #6750A4',
        boxShadow: 'none',
      },
      ghost: {
        background: 'transparent',
        backgroundHover: 'rgba(0, 0, 0, 0.04)',
        backgroundActive: 'rgba(0, 0, 0, 0.08)',
        color: '#49454F',
        border: 'none',
        boxShadow: 'none',
      },
      danger: {
        background: '#DC3545',
        backgroundHover: '#C82333',
        backgroundActive: '#BD2130',
        color: '#FFFFFF',
        border: 'none',
        boxShadow: '0 2px 4px rgba(220, 53, 69, 0.2)',
      },
    },

    // Card
    card: {
      background: '#FFFFFF',
      border: '1px solid #E7E0EC',
      borderRadius: '12px',
      boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1), 0 1px 2px rgba(0, 0, 0, 0.06)',
      padding: '1.5rem',
    },

    // Input
    input: {
      background: '#FFFFFF',
      border: '1px solid #CAC4D0',
      borderFocus: '2px solid #6750A4',
      borderError: '2px solid #DC3545',
      borderRadius: '4px',
      padding: '0.75rem 1rem',
      color: '#1C1B1F',
      placeholder: '#79747E',
    },
  },

  // Accessibility
  accessibility: {
    contrastRatio: 7.0, // WCAG AA
    colorblindSafe: true,
    reducedMotion: true,
    focusVisible: {
      outlineColor: '#6750A4',
      outlineWidth: '2px',
      outlineOffset: '2px',
    },
  },
});

export default professionalTheme;
