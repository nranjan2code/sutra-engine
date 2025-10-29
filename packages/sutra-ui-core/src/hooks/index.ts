/**
 * Sutra UI Core - Utility Hooks
 * 
 * React hooks for responsive design, accessibility, and theme utilities.
 */

import { useEffect, useState, useMemo } from 'react';
import { useTheme } from '../theme/ThemeProvider';

// ============================================================================
// useMediaQuery Hook
// ============================================================================

/**
 * Hook to match media queries
 * 
 * @example
 * ```typescript
 * const isMobile = useMediaQuery('(max-width: 768px)');
 * ```
 */
export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState<boolean>(() => {
    if (typeof window !== 'undefined') {
      return window.matchMedia(query).matches;
    }
    return false;
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;
    
    const mediaQuery = window.matchMedia(query);
    const handler = (e: MediaQueryListEvent) => setMatches(e.matches);
    
    // Modern browsers
    if (mediaQuery.addEventListener) {
      mediaQuery.addEventListener('change', handler);
      return () => mediaQuery.removeEventListener('change', handler);
    }
    // Legacy browsers
    else {
      mediaQuery.addListener(handler);
      return () => mediaQuery.removeListener(handler);
    }
  }, [query]);

  return matches;
}

// ============================================================================
// useBreakpoint Hook
// ============================================================================

export type Breakpoint = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';

/**
 * Hook to get current breakpoint
 * 
 * @example
 * ```typescript
 * const breakpoint = useBreakpoint();
 * const isMobile = breakpoint === 'xs' || breakpoint === 'sm';
 * ```
 */
export function useBreakpoint(): Breakpoint {
  const { theme } = useTheme();
  const breakpoints = theme.tokens.breakpoints;
  
  const is2xl = useMediaQuery(`(min-width: ${breakpoints['2xl']})`);
  const isXl = useMediaQuery(`(min-width: ${breakpoints.xl})`);
  const isLg = useMediaQuery(`(min-width: ${breakpoints.lg})`);
  const isMd = useMediaQuery(`(min-width: ${breakpoints.md})`);
  const isSm = useMediaQuery(`(min-width: ${breakpoints.sm})`);
  
  if (is2xl) return '2xl';
  if (isXl) return 'xl';
  if (isLg) return 'lg';
  if (isMd) return 'md';
  if (isSm) return 'sm';
  return 'xs';
}

// ============================================================================
// useReducedMotion Hook
// ============================================================================

/**
 * Hook to detect user's motion preference
 * 
 * @example
 * ```typescript
 * const prefersReducedMotion = useReducedMotion();
 * const animationDuration = prefersReducedMotion ? '0ms' : '250ms';
 * ```
 */
export function useReducedMotion(): boolean {
  return useMediaQuery('(prefers-reduced-motion: reduce)');
}

// ============================================================================
// useFocusVisible Hook
// ============================================================================

/**
 * Hook to detect if focus should be visible (keyboard navigation)
 * 
 * @example
 * ```typescript
 * const focusVisible = useFocusVisible();
 * ```
 */
export function useFocusVisible(): boolean {
  const [hadKeyboardEvent, setHadKeyboardEvent] = useState(false);
  const [focusVisible, setFocusVisible] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Tab') {
        setHadKeyboardEvent(true);
      }
    };

    const handleMouseDown = () => {
      setHadKeyboardEvent(false);
    };

    const handleFocus = () => {
      if (hadKeyboardEvent) {
        setFocusVisible(true);
      }
    };

    const handleBlur = () => {
      setFocusVisible(false);
    };

    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('mousedown', handleMouseDown);
    document.addEventListener('focus', handleFocus, true);
    document.addEventListener('blur', handleBlur, true);

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      document.removeEventListener('mousedown', handleMouseDown);
      document.removeEventListener('focus', handleFocus, true);
      document.removeEventListener('blur', handleBlur, true);
    };
  }, [hadKeyboardEvent]);

  return focusVisible;
}

// ============================================================================
// useColorScheme Hook
// ============================================================================

export type ColorScheme = 'light' | 'dark';

/**
 * Hook to detect system color scheme preference
 * 
 * @example
 * ```typescript
 * const colorScheme = useColorScheme();
 * ```
 */
export function useColorScheme(): ColorScheme {
  const prefersDark = useMediaQuery('(prefers-color-scheme: dark)');
  return prefersDark ? 'dark' : 'light';
}

// ============================================================================
// useThemeColor Hook
// ============================================================================

/**
 * Hook to get a specific color token from the current theme
 * 
 * @example
 * ```typescript
 * const primaryColor = useThemeColor('primary');
 * const backgroundColor = useThemeColor('background');
 * ```
 */
export function useThemeColor(colorKey: string): string {
  const { theme } = useTheme();
  
  return useMemo(() => {
    const keys = colorKey.split('.');
    let value: any = theme.tokens.color;
    
    for (const key of keys) {
      if (value && typeof value === 'object') {
        value = value[key];
      } else {
        return '#000000'; // Fallback
      }
    }
    
    return typeof value === 'string' ? value : '#000000';
  }, [theme, colorKey]);
}

// ============================================================================
// useResponsiveValue Hook
// ============================================================================

/**
 * Hook to get responsive values based on breakpoint
 * 
 * @example
 * ```typescript
 * const columns = useResponsiveValue({ xs: 1, sm: 2, md: 3, lg: 4 });
 * ```
 */
export function useResponsiveValue<T>(values: Partial<Record<Breakpoint, T>>): T | undefined {
  const breakpoint = useBreakpoint();
  
  return useMemo(() => {
    const breakpointOrder: Breakpoint[] = ['2xl', 'xl', 'lg', 'md', 'sm', 'xs'];
    const currentIndex = breakpointOrder.indexOf(breakpoint);
    
    for (let i = currentIndex; i < breakpointOrder.length; i++) {
      const bp = breakpointOrder[i];
      if (values[bp] !== undefined) {
        return values[bp];
      }
    }
    
    return undefined;
  }, [breakpoint, values]);
}

// ============================================================================
// useViewportSize Hook
// ============================================================================

export interface ViewportSize {
  width: number;
  height: number;
}

/**
 * Hook to get current viewport dimensions
 * 
 * @example
 * ```typescript
 * const { width, height } = useViewportSize();
 * ```
 */
export function useViewportSize(): ViewportSize {
  const [size, setSize] = useState<ViewportSize>(() => {
    if (typeof window !== 'undefined') {
      return {
        width: window.innerWidth,
        height: window.innerHeight,
      };
    }
    return { width: 0, height: 0 };
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;

    const handleResize = () => {
      setSize({
        width: window.innerWidth,
        height: window.innerHeight,
      });
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  return size;
}

// ============================================================================
// useContrastText Hook
// ============================================================================

/**
 * Hook to get contrasting text color for a background
 * 
 * @example
 * ```typescript
 * const textColor = useContrastText('#000000'); // Returns light color
 * const textColor2 = useContrastText('#ffffff'); // Returns dark color
 * ```
 */
export function useContrastText(backgroundColor: string): string {
  const { theme } = useTheme();
  
  return useMemo(() => {
    const luminance = getRelativeLuminance(backgroundColor);
    const threshold = 0.5;
    
    return luminance > threshold 
      ? theme.tokens.color.text.primary // Dark text for light backgrounds
      : theme.tokens.color.text.inverse || '#ffffff'; // Light text for dark backgrounds
  }, [backgroundColor, theme]);
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Calculate relative luminance of a color (WCAG 2.0)
 */
function getRelativeLuminance(color: string): number {
  const hex = color.replace('#', '');
  const r = parseInt(hex.substr(0, 2), 16) / 255;
  const g = parseInt(hex.substr(2, 2), 16) / 255;
  const b = parseInt(hex.substr(4, 2), 16) / 255;
  
  const rsrgb = r <= 0.03928 ? r / 12.92 : Math.pow((r + 0.055) / 1.055, 2.4);
  const gsrgb = g <= 0.03928 ? g / 12.92 : Math.pow((g + 0.055) / 1.055, 2.4);
  const bsrgb = b <= 0.03928 ? b / 12.92 : Math.pow((b + 0.055) / 1.055, 2.4);
  
  return 0.2126 * rsrgb + 0.7152 * gsrgb + 0.0722 * bsrgb;
}
