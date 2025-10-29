/**
 * Sutra UI Core - Theme Provider
 * 
 * React context provider for theme management.
 * Handles theme switching, CSS variable injection, and theme persistence.
 */

import React, { createContext, useContext, useEffect, useMemo, useState } from 'react';
import type { Theme, ThemeContextValue } from './types';

// ============================================================================
// Theme Context
// ============================================================================

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

// ============================================================================
// Theme Provider Component
// ============================================================================

export interface ThemeProviderProps {
  /** Initial theme */
  theme: Theme;
  
  /** Available themes for switching */
  availableThemes?: Theme[];
  
  /** Persist theme preference to localStorage */
  persist?: boolean;
  
  /** localStorage key for theme persistence */
  storageKey?: string;
  
  /** Children components */
  children: React.ReactNode;
}

export function ThemeProvider({
  theme: initialTheme,
  availableThemes = [],
  persist = false,
  storageKey = 'sutra-ui-theme',
  children,
}: ThemeProviderProps): React.ReactElement {
  // ============================================================================
  // State Management
  // ============================================================================
  
  const [currentTheme, setCurrentTheme] = useState<Theme>(() => {
    if (persist && typeof window !== 'undefined') {
      const stored = localStorage.getItem(storageKey);
      if (stored) {
        const foundTheme = availableThemes.find(t => t.name === stored);
        if (foundTheme) return foundTheme;
      }
    }
    return initialTheme;
  });

  // ============================================================================
  // Theme Switching
  // ============================================================================
  
  const setTheme = useMemo(() => {
    return (theme: Theme | string) => {
      const newTheme = typeof theme === 'string'
        ? availableThemes.find(t => t.name === theme) || currentTheme
        : theme;
      
      setCurrentTheme(newTheme);
      
      if (persist && typeof window !== 'undefined') {
        localStorage.setItem(storageKey, newTheme.name);
      }
    };
  }, [availableThemes, currentTheme, persist, storageKey]);

  // ============================================================================
  // CSS Variable Injection
  // ============================================================================
  
  useEffect(() => {
    if (typeof window === 'undefined') return;
    
    const root = document.documentElement;
    const { tokens } = currentTheme;
    
    // Color tokens
    root.style.setProperty('--sutra-color-primary', tokens.color.primary);
    root.style.setProperty('--sutra-color-secondary', tokens.color.secondary);
    root.style.setProperty('--sutra-color-success', tokens.color.success);
    root.style.setProperty('--sutra-color-warning', tokens.color.warning);
    root.style.setProperty('--sutra-color-error', tokens.color.error);
    root.style.setProperty('--sutra-color-info', tokens.color.info);
    
    root.style.setProperty('--sutra-color-surface', tokens.color.surface);
    root.style.setProperty('--sutra-color-background', tokens.color.background);
    
    root.style.setProperty('--sutra-color-text-primary', tokens.color.text.primary);
    root.style.setProperty('--sutra-color-text-secondary', tokens.color.text.secondary);
    root.style.setProperty('--sutra-color-text-disabled', tokens.color.text.disabled);
    
    root.style.setProperty('--sutra-color-border', tokens.color.border.default);
    
    // Typography tokens
    root.style.setProperty('--sutra-font-family-base', tokens.typography.fontFamily.base);
    root.style.setProperty('--sutra-font-size-base', tokens.typography.fontSize.base);
    root.style.setProperty('--sutra-font-weight-normal', String(tokens.typography.fontWeight.normal));
    root.style.setProperty('--sutra-line-height-normal', String(tokens.typography.lineHeight.normal));
    
    // Spacing tokens
    root.style.setProperty('--sutra-spacing-base', `${tokens.spacing.base}px`);
    
    // Shape tokens
    root.style.setProperty('--sutra-border-radius-base', tokens.shape.borderRadius.base);
    root.style.setProperty('--sutra-border-width-base', tokens.shape.borderWidth.base);
    
    // Animation tokens
    root.style.setProperty('--sutra-duration-normal', tokens.animation.duration.normal);
    root.style.setProperty('--sutra-easing-ease-in-out', tokens.animation.easing.easeInOut);
    
    // Custom CSS variables
    if (currentTheme.cssVariables) {
      Object.entries(currentTheme.cssVariables).forEach(([key, value]) => {
        root.style.setProperty(key, value);
      });
    }
    
    // Set data attribute for theme-specific styles
    root.setAttribute('data-sutra-theme', currentTheme.name);
    
    // Set color scheme for native browser UI
    const isDark = isColorDark(tokens.color.background);
    root.style.setProperty('color-scheme', isDark ? 'dark' : 'light');
    
  }, [currentTheme]);

  // ============================================================================
  // Context Value
  // ============================================================================
  
  const contextValue = useMemo<ThemeContextValue>(() => ({
    theme: currentTheme,
    setTheme,
    availableThemes: availableThemes.length > 0 ? availableThemes : undefined,
  }), [currentTheme, setTheme, availableThemes]);

  // ============================================================================
  // Render
  // ============================================================================
  
  return (
    <ThemeContext.Provider value={contextValue}>
      {children}
    </ThemeContext.Provider>
  );
}

// ============================================================================
// useTheme Hook
// ============================================================================

export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  
  return context;
}

// ============================================================================
// Utilities
// ============================================================================

/**
 * Determine if a color is dark or light
 * Uses relative luminance calculation (WCAG 2.0)
 */
function isColorDark(color: string): boolean {
  // Convert hex to RGB
  const hex = color.replace('#', '');
  const r = parseInt(hex.substr(0, 2), 16) / 255;
  const g = parseInt(hex.substr(2, 2), 16) / 255;
  const b = parseInt(hex.substr(4, 2), 16) / 255;
  
  // Calculate relative luminance
  const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  
  return luminance < 0.5;
}
