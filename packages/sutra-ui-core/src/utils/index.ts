/**
 * Sutra UI Core - Utility Functions
 * 
 * Helper functions for styling, colors, and accessibility.
 */

// ============================================================================
// CSS Class Name Utilities
// ============================================================================

/**
 * Combine class names conditionally
 * 
 * @example
 * ```typescript
 * cn('base', isActive && 'active', 'extra')
 * // Returns: 'base active extra'
 * ```
 */
export function cn(...inputs: (string | boolean | undefined | null)[]): string {
  return inputs.filter(Boolean).join(' ');
}

/**
 * Create BEM-style class names
 * 
 * @example
 * ```typescript
 * bem('button', 'primary', { loading: true, disabled: false })
 * // Returns: 'button button--primary button--loading'
 * ```
 */
export function bem(
  block: string,
  element?: string,
  modifiers?: Record<string, boolean | string | undefined>
): string {
  const base = element ? `${block}__${element}` : block;
  const classes = [base];

  if (modifiers) {
    Object.entries(modifiers).forEach(([key, value]) => {
      if (value === true) {
        classes.push(`${base}--${key}`);
      } else if (typeof value === 'string') {
        classes.push(`${base}--${key}-${value}`);
      }
    });
  }

  return classes.join(' ');
}

// ============================================================================
// Color Utilities
// ============================================================================

/**
 * Convert hex to RGBA
 * 
 * @example
 * ```typescript
 * hexToRgba('#ff0000', 0.5) // Returns: 'rgba(255, 0, 0, 0.5)'
 * ```
 */
export function hexToRgba(hex: string, alpha: number = 1): string {
  const cleaned = hex.replace('#', '');
  const r = parseInt(cleaned.substr(0, 2), 16);
  const g = parseInt(cleaned.substr(2, 2), 16);
  const b = parseInt(cleaned.substr(4, 2), 16);
  
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

/**
 * Lighten a color by a percentage
 * 
 * @example
 * ```typescript
 * lighten('#000000', 0.2) // Returns: '#333333'
 * ```
 */
export function lighten(color: string, amount: number): string {
  const hex = color.replace('#', '');
  const num = parseInt(hex, 16);
  
  let r = (num >> 16) + Math.round(255 * amount);
  let g = ((num >> 8) & 0x00FF) + Math.round(255 * amount);
  let b = (num & 0x0000FF) + Math.round(255 * amount);
  
  r = Math.min(255, Math.max(0, r));
  g = Math.min(255, Math.max(0, g));
  b = Math.min(255, Math.max(0, b));
  
  return '#' + (r * 0x10000 + g * 0x100 + b).toString(16).padStart(6, '0');
}

/**
 * Darken a color by a percentage
 * 
 * @example
 * ```typescript
 * darken('#ffffff', 0.2) // Returns: '#cccccc'
 * ```
 */
export function darken(color: string, amount: number): string {
  return lighten(color, -amount);
}

/**
 * Calculate contrast ratio between two colors (WCAG 2.0)
 * 
 * @example
 * ```typescript
 * contrastRatio('#000000', '#ffffff') // Returns: 21 (maximum contrast)
 * ```
 */
export function contrastRatio(color1: string, color2: string): number {
  const l1 = getRelativeLuminance(color1);
  const l2 = getRelativeLuminance(color2);
  
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  
  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Check if a color combination meets WCAG AA standards
 * 
 * @example
 * ```typescript
 * isAccessible('#000000', '#ffffff') // Returns: true
 * ```
 */
export function isAccessible(
  foreground: string,
  background: string,
  level: 'AA' | 'AAA' = 'AA',
  size: 'normal' | 'large' = 'normal'
): boolean {
  const ratio = contrastRatio(foreground, background);
  
  if (level === 'AAA') {
    return size === 'large' ? ratio >= 4.5 : ratio >= 7.0;
  }
  
  return size === 'large' ? ratio >= 3.0 : ratio >= 4.5;
}

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

// ============================================================================
// Animation Utilities
// ============================================================================

/**
 * Create CSS transition string
 * 
 * @example
 * ```typescript
 * transition(['opacity', 'transform'], '250ms', 'ease-in-out')
 * // Returns: 'opacity 250ms ease-in-out, transform 250ms ease-in-out'
 * ```
 */
export function transition(
  properties: string | string[],
  duration: string = '250ms',
  easing: string = 'ease-in-out'
): string {
  const props = Array.isArray(properties) ? properties : [properties];
  return props.map(prop => `${prop} ${duration} ${easing}`).join(', ');
}

// ============================================================================
// Layout Utilities
// ============================================================================

/**
 * Convert spacing scale to actual value
 * 
 * @example
 * ```typescript
 * spacing(4) // Returns: '1rem'
 * spacing(8) // Returns: '2rem'
 * ```
 */
export function spacing(value: number, base: number = 4): string {
  return `${(value * base) / 16}rem`;
}

/**
 * Clamp a value between min and max
 * 
 * @example
 * ```typescript
 * clamp(5, 0, 10) // Returns: 5
 * clamp(-5, 0, 10) // Returns: 0
 * clamp(15, 0, 10) // Returns: 10
 * ```
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

/**
 * Convert pixel value to rem
 * 
 * @example
 * ```typescript
 * pxToRem(16) // Returns: '1rem'
 * pxToRem(24) // Returns: '1.5rem'
 * ```
 */
export function pxToRem(px: number, baseFontSize: number = 16): string {
  return `${px / baseFontSize}rem`;
}

/**
 * Convert rem value to pixels
 * 
 * @example
 * ```typescript
 * remToPx(1) // Returns: 16
 * remToPx(1.5) // Returns: 24
 * ```
 */
export function remToPx(rem: number, baseFontSize: number = 16): number {
  return rem * baseFontSize;
}

// ============================================================================
// Accessibility Utilities
// ============================================================================

/**
 * Generate accessible label for screen readers
 * 
 * @example
 * ```typescript
 * srOnly('Close modal')
 * // Returns CSS that hides element visually but keeps it for screen readers
 * ```
 */
export function srOnly(): React.CSSProperties {
  return {
    position: 'absolute',
    width: '1px',
    height: '1px',
    padding: '0',
    margin: '-1px',
    overflow: 'hidden',
    clip: 'rect(0, 0, 0, 0)',
    whiteSpace: 'nowrap',
    borderWidth: '0',
  };
}

/**
 * Check if element has visible focus
 * 
 * @example
 * ```typescript
 * focusVisible({ outline: '2px solid blue' })
 * // Returns focus styles only for keyboard navigation
 * ```
 */
export function focusVisible(styles: React.CSSProperties): string {
  return `
    &:focus {
      outline: none;
    }
    &:focus-visible {
      ${Object.entries(styles).map(([key, value]) => `${key}: ${value};`).join('\n')}
    }
  `;
}

// ============================================================================
// Type Guards
// ============================================================================

/**
 * Check if value is a valid hex color
 */
export function isHexColor(value: string): boolean {
  return /^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})$/.test(value);
}

/**
 * Check if value is a valid CSS length
 */
export function isCSSLength(value: string): boolean {
  return /^-?\d+(\.\d+)?(px|em|rem|%|vh|vw|vmin|vmax)$/.test(value);
}
