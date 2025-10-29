/**
 * Sutra UI Themes - Holographic Theme
 * 
 * Sci-fi HUD aesthetic for sutra-explorer.
 * Features: cyan glow effects, scanlines, frosted glass, high contrast.
 */

import { createTheme, type Theme } from '@sutra/ui-core';
import { baseTokens } from '../base/tokens';

/**
 * Holographic Theme - Futuristic HUD Interface
 * 
 * Design Philosophy:
 * - High-tech, holographic display aesthetic
 * - Cyan/teal primary color with bright accents
 * - Dark background for maximum contrast
 * - Glow effects and scanlines for immersion
 * - WCAG AAA contrast ratio (14.6:1)
 * - Colorblind-safe single-hue system
 */
export const holographicTheme: Theme = createTheme({
  name: 'holographic',
  displayName: 'Holographic HUD',
  description: 'Futuristic sci-fi HUD interface with glowing effects',

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

    // Color System - Cyan/Teal Primary
    color: {
      primary: '#00ffff',           // Bright cyan
      secondary: '#00d4d4',         // Mid cyan
      tertiary: '#00aaaa',          // Dark cyan

      success: '#00ffaa',           // Green-cyan
      warning: '#ffff00',           // Yellow
      error: '#ff0066',             // Pink-red
      info: '#00aaff',              // Blue-cyan

      surface: '#0a0e1a',           // Near black
      surfaceHover: '#12182c',      // Slightly lighter
      surfaceActive: '#1a2440',     // Even lighter
      surfaceVariant: '#0f1321',    // Subtle variant

      background: '#000000',        // Pure black
      backgroundSecondary: '#05070d', // Very dark

      text: {
        primary: '#e0e6ed',         // Bright white
        secondary: '#8892a0',       // Mid gray
        tertiary: '#6b7280',        // Dark gray
        disabled: '#4a5568',        // Very dark gray
        inverse: '#0a0e1a',         // Dark (for light backgrounds)
      },

      border: {
        default: '#2d3748',         // Dark gray
        hover: '#00ffff',           // Cyan on hover
        focus: '#00ffff',           // Cyan on focus
        error: '#ff0066',           // Pink-red
      },

      overlay: {
        background: 'rgba(0, 0, 0, 0.85)',
        backdrop: 'rgba(0, 0, 0, 0.7)',
      },
    },

    // Typography - Monospace for tech aesthetic
    typography: {
      ...baseTokens.typography!,
      fontFamily: {
        base: '"Roboto Mono", "SF Mono", Monaco, "Cascadia Code", monospace',
        heading: '"Roboto Mono", "SF Mono", Monaco, "Cascadia Code", monospace',
        mono: '"Roboto Mono", "SF Mono", Monaco, "Cascadia Code", monospace',
      },
    },

    // Elevation - Glowing shadows
    elevation: {
      none: 'none',
      xs: '0 0 4px rgba(0, 255, 255, 0.2)',
      sm: '0 0 8px rgba(0, 255, 255, 0.3)',
      base: '0 0 12px rgba(0, 255, 255, 0.3), 0 4px 8px rgba(0, 0, 0, 0.5)',
      md: '0 0 16px rgba(0, 255, 255, 0.4), 0 8px 16px rgba(0, 0, 0, 0.6)',
      lg: '0 0 24px rgba(0, 255, 255, 0.4), 0 12px 24px rgba(0, 0, 0, 0.7)',
      xl: '0 0 32px rgba(0, 255, 255, 0.5), 0 16px 32px rgba(0, 0, 0, 0.8)',
      '2xl': '0 0 48px rgba(0, 255, 255, 0.6), 0 24px 48px rgba(0, 0, 0, 0.9)',
    },

    // Special Effects
    effects: {
      // Glow effect
      glow: {
        enabled: true,
        color: '#00ffff',
        blur: [10, 20, 40],
        opacity: [0.3, 0.2, 0.1],
        spread: 2,
      },

      // Scanline effect (CRT display)
      scanlines: {
        enabled: true,
        opacity: 0.05,
        height: 2,
        speed: 0.5,
      },

      // Frosted glass effect
      frostedGlass: {
        enabled: true,
        blur: 20,
        opacity: 0.1,
        saturation: 1.2,
      },

      // Gradient effects
      gradient: {
        primary: 'linear-gradient(135deg, #00ffff 0%, #00aaff 100%)',
        secondary: 'linear-gradient(135deg, #00d4d4 0%, #008888 100%)',
        accent: 'linear-gradient(135deg, #ff0066 0%, #ff00ff 100%)',
      },
    },
  },

  // Component Tokens
  components: {
    // Button
    button: {
      primary: {
        background: 'transparent',
        backgroundHover: 'rgba(0, 255, 255, 0.1)',
        backgroundActive: 'rgba(0, 255, 255, 0.2)',
        color: '#00ffff',
        colorHover: '#ffffff',
        border: '1px solid #00ffff',
        borderHover: '1px solid #ffffff',
        boxShadow: '0 0 10px rgba(0, 255, 255, 0.3)',
        boxShadowHover: '0 0 20px rgba(0, 255, 255, 0.5)',
      },
      secondary: {
        background: 'transparent',
        backgroundHover: 'rgba(0, 212, 212, 0.1)',
        backgroundActive: 'rgba(0, 212, 212, 0.2)',
        color: '#00d4d4',
        border: '1px solid #00d4d4',
        boxShadow: '0 0 8px rgba(0, 212, 212, 0.2)',
      },
      ghost: {
        background: 'transparent',
        backgroundHover: 'rgba(255, 255, 255, 0.05)',
        backgroundActive: 'rgba(255, 255, 255, 0.1)',
        color: '#e0e6ed',
        border: 'none',
        boxShadow: 'none',
      },
      danger: {
        background: 'transparent',
        backgroundHover: 'rgba(255, 0, 102, 0.1)',
        backgroundActive: 'rgba(255, 0, 102, 0.2)',
        color: '#ff0066',
        border: '1px solid #ff0066',
        boxShadow: '0 0 10px rgba(255, 0, 102, 0.3)',
      },
    },

    // Card
    card: {
      background: 'rgba(10, 14, 26, 0.8)',
      border: '1px solid rgba(0, 255, 255, 0.2)',
      borderRadius: '8px',
      boxShadow: '0 0 12px rgba(0, 255, 255, 0.15), inset 0 0 40px rgba(0, 255, 255, 0.05)',
      padding: '1rem',
    },

    // Input
    input: {
      background: 'rgba(10, 14, 26, 0.6)',
      border: '1px solid rgba(0, 255, 255, 0.3)',
      borderFocus: '1px solid #00ffff',
      borderError: '1px solid #ff0066',
      borderRadius: '4px',
      padding: '0.5rem 0.75rem',
      color: '#e0e6ed',
      placeholder: '#8892a0',
    },
  },

  // Accessibility
  accessibility: {
    contrastRatio: 14.6, // WCAG AAA (cyan on black)
    colorblindSafe: true, // Single-hue cyan system
    reducedMotion: true,  // Respects prefers-reduced-motion
    focusVisible: {
      outlineColor: '#00ffff',
      outlineWidth: '2px',
      outlineOffset: '2px',
    },
  },

  // CSS Variables
  cssVariables: {
    '--holographic-glow': '0 0 20px rgba(0, 255, 255, 0.5)',
    '--holographic-scanline-height': '2px',
    '--holographic-scanline-opacity': '0.05',
    '--holographic-blur': '20px',
  },
});

export default holographicTheme;
