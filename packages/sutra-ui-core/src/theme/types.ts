/**
 * Sutra UI Core - Theme Type Definitions
 * 
 * Complete TypeScript definitions for the theme system.
 * Provides type safety across all theme tokens and configurations.
 */

// ============================================================================
// Color Tokens
// ============================================================================

export interface ColorTokens {
  /** Primary brand color */
  primary: string;
  /** Secondary brand color */
  secondary: string;
  /** Tertiary accent color */
  tertiary?: string;
  
  /** Success state color */
  success: string;
  /** Warning state color */
  warning: string;
  /** Error state color */
  error: string;
  /** Info state color */
  info: string;
  
  /** Surface background color (cards, panels) */
  surface: string;
  /** Surface hover state */
  surfaceHover?: string;
  /** Surface active/pressed state */
  surfaceActive?: string;
  /** Surface variant for subtle differentiation */
  surfaceVariant?: string;
  
  /** Main background color */
  background: string;
  /** Secondary background color */
  backgroundSecondary?: string;
  
  /** Text colors */
  text: {
    primary: string;
    secondary: string;
    tertiary?: string;
    disabled: string;
    inverse?: string;
  };
  
  /** Border colors */
  border: {
    default: string;
    hover?: string;
    focus?: string;
    error?: string;
  };
  
  /** Overlay colors (modals, tooltips) */
  overlay?: {
    background: string;
    backdrop: string;
  };
}

// ============================================================================
// Typography Tokens
// ============================================================================

export interface TypographyTokens {
  /** Font family stack */
  fontFamily: {
    base: string;
    heading?: string;
    mono?: string;
  };
  
  /** Font sizes */
  fontSize: {
    xs: string;
    sm: string;
    base: string;
    lg: string;
    xl: string;
    '2xl': string;
    '3xl': string;
    '4xl': string;
  };
  
  /** Font weights */
  fontWeight: {
    light: number;
    normal: number;
    medium: number;
    semibold: number;
    bold: number;
  };
  
  /** Line heights */
  lineHeight: {
    tight: number;
    normal: number;
    relaxed: number;
    loose: number;
  };
  
  /** Letter spacing */
  letterSpacing?: {
    tight: string;
    normal: string;
    wide: string;
  };
}

// ============================================================================
// Spacing Tokens
// ============================================================================

export interface SpacingTokens {
  /** Base spacing unit (typically 4px or 8px) */
  base: number;
  
  /** Spacing scale */
  scale: {
    '0': string;
    '1': string;
    '2': string;
    '3': string;
    '4': string;
    '5': string;
    '6': string;
    '8': string;
    '10': string;
    '12': string;
    '16': string;
    '20': string;
    '24': string;
    '32': string;
  };
}

// ============================================================================
// Shape Tokens
// ============================================================================

export interface ShapeTokens {
  /** Border radius values */
  borderRadius: {
    none: string;
    sm: string;
    base: string;
    md: string;
    lg: string;
    xl: string;
    '2xl': string;
    full: string;
  };
  
  /** Border widths */
  borderWidth: {
    none: string;
    thin: string;
    base: string;
    thick: string;
  };
}

// ============================================================================
// Elevation/Shadow Tokens
// ============================================================================

export interface ElevationTokens {
  none: string;
  xs: string;
  sm: string;
  base: string;
  md: string;
  lg: string;
  xl: string;
  '2xl': string;
}

// ============================================================================
// Animation Tokens
// ============================================================================

export interface AnimationTokens {
  /** Animation durations */
  duration: {
    instant: string;
    fast: string;
    normal: string;
    slow: string;
    slower: string;
  };
  
  /** Easing functions */
  easing: {
    linear: string;
    easeIn: string;
    easeOut: string;
    easeInOut: string;
    spring: string;
  };
}

// ============================================================================
// Breakpoint Tokens
// ============================================================================

export interface BreakpointTokens {
  xs: string;
  sm: string;
  md: string;
  lg: string;
  xl: string;
  '2xl': string;
}

// ============================================================================
// Z-Index Tokens
// ============================================================================

export interface ZIndexTokens {
  base: number;
  dropdown: number;
  sticky: number;
  fixed: number;
  modalBackdrop: number;
  modal: number;
  popover: number;
  tooltip: number;
}

// ============================================================================
// Effect Tokens (Theme-Specific)
// ============================================================================

export interface EffectTokens {
  /** Glow effect configuration */
  glow?: {
    enabled: boolean;
    color?: string;
    blur?: number[];
    opacity?: number[];
    spread?: number;
  };
  
  /** Scanline effect (holographic theme) */
  scanlines?: {
    enabled: boolean;
    opacity?: number;
    height?: number;
    speed?: number;
  };
  
  /** Frosted glass/blur effect */
  frostedGlass?: {
    enabled: boolean;
    blur?: number;
    opacity?: number;
    saturation?: number;
  };
  
  /** Gradient effects */
  gradient?: {
    primary?: string;
    secondary?: string;
    accent?: string;
  };
}

// ============================================================================
// Component-Specific Tokens
// ============================================================================

export interface ButtonTokens {
  primary: ComponentVariantTokens;
  secondary: ComponentVariantTokens;
  ghost: ComponentVariantTokens;
  danger: ComponentVariantTokens;
}

export interface ComponentVariantTokens {
  background: string;
  backgroundHover?: string;
  backgroundActive?: string;
  color: string;
  colorHover?: string;
  border?: string;
  borderHover?: string;
  boxShadow?: string;
  boxShadowHover?: string;
}

export interface CardTokens {
  background: string;
  border?: string;
  borderRadius: string;
  boxShadow?: string;
  padding: string;
}

export interface InputTokens {
  background: string;
  border: string;
  borderFocus: string;
  borderError: string;
  borderRadius: string;
  padding: string;
  color: string;
  placeholder: string;
}

export interface ComponentTokens {
  button: ButtonTokens;
  card: CardTokens;
  input: InputTokens;
  // Add more component tokens as needed
}

// ============================================================================
// Accessibility Configuration
// ============================================================================

export interface AccessibilityConfig {
  /** WCAG contrast ratio (4.5 for AA, 7.0 for AAA) */
  contrastRatio: number;
  
  /** Whether theme is safe for colorblind users */
  colorblindSafe: boolean;
  
  /** Reduced motion support */
  reducedMotion: boolean;
  
  /** Focus visible configuration */
  focusVisible?: {
    outlineColor: string;
    outlineWidth: string;
    outlineOffset: string;
  };
}

// ============================================================================
// Main Theme Interface
// ============================================================================

export interface ThemeTokens {
  color: ColorTokens;
  typography: TypographyTokens;
  spacing: SpacingTokens;
  shape: ShapeTokens;
  elevation: ElevationTokens;
  animation: AnimationTokens;
  breakpoints: BreakpointTokens;
  zIndex: ZIndexTokens;
  effects?: EffectTokens;
}

export interface Theme {
  /** Unique theme identifier */
  name: string;
  
  /** Human-readable theme name */
  displayName: string;
  
  /** Theme description */
  description?: string;
  
  /** Design tokens */
  tokens: ThemeTokens;
  
  /** Component-specific styling */
  components: ComponentTokens;
  
  /** Accessibility configuration */
  accessibility: AccessibilityConfig;
  
  /** Custom CSS variables (optional) */
  cssVariables?: Record<string, string>;
}

// ============================================================================
// Theme Creation Options
// ============================================================================

export interface CreateThemeOptions {
  /** Base theme to extend (optional) */
  baseTheme?: Theme;
  
  /** Token overrides */
  tokens?: Partial<ThemeTokens>;
  
  /** Component token overrides */
  components?: Partial<ComponentTokens>;
  
  /** Accessibility overrides */
  accessibility?: Partial<AccessibilityConfig>;
}

// ============================================================================
// Theme Context Types
// ============================================================================

export interface ThemeContextValue {
  /** Current theme */
  theme: Theme;
  
  /** Switch to a different theme */
  setTheme: (theme: Theme | string) => void;
  
  /** Available themes */
  availableThemes?: Theme[];
}
