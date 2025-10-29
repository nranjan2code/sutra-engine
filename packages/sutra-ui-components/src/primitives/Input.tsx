/**
 * Sutra UI Components - Input
 * 
 * Accessible input component with validation, error states, and helper text.
 * Performance-optimized with React.memo and useMemo.
 */

import React, { forwardRef, useMemo, memo, useState, useCallback } from 'react';
import { useTheme, cn } from '@sutra/ui-core';

export interface InputProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'size'> {
  /** Input label */
  label?: string;
  
  /** Helper text displayed below input */
  helperText?: string;
  
  /** Error message (shows error state when present) */
  error?: string;
  
  /** Input size */
  size?: 'sm' | 'md' | 'lg';
  
  /** Input variant */
  variant?: 'outlined' | 'filled' | 'unstyled';
  
  /** Full width */
  fullWidth?: boolean;
  
  /** Icon element to display at start */
  startIcon?: React.ReactNode;
  
  /** Icon element to display at end */
  endIcon?: React.ReactNode;
  
  /** Loading state */
  loading?: boolean;
  
  /** Custom validation function */
  validate?: (value: string) => string | undefined;
  
  /** Callback when validation changes */
  onValidation?: (error: string | undefined) => void;
}

const InputComponent = forwardRef<HTMLInputElement, InputProps>(
  (
    {
      label,
      helperText,
      error: externalError,
      size = 'md',
      variant = 'outlined',
      fullWidth = false,
      startIcon,
      endIcon,
      loading = false,
      validate,
      onValidation,
      className,
      disabled,
      required,
      onChange,
      onBlur,
      ...props
    },
    ref
  ) => {
    const { theme } = useTheme();
    const [internalError, setInternalError] = useState<string | undefined>();
    const [isFocused, setIsFocused] = useState(false);
    const [isTouched, setIsTouched] = useState(false);
    
    // Use external error if provided, otherwise use internal validation error
    const error = externalError || (isTouched ? internalError : undefined);
    const hasError = Boolean(error);
    
    // Validation handler
    const handleValidation = useCallback((value: string) => {
      if (!validate) return;
      
      const validationError = validate(value);
      setInternalError(validationError);
      onValidation?.(validationError);
    }, [validate, onValidation]);
    
    // Change handler with validation
    const handleChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
      onChange?.(e);
      if (isTouched) {
        handleValidation(e.target.value);
      }
    }, [onChange, isTouched, handleValidation]);
    
    // Blur handler
    const handleBlur = useCallback((e: React.FocusEvent<HTMLInputElement>) => {
      setIsFocused(false);
      setIsTouched(true);
      handleValidation(e.target.value);
      onBlur?.(e);
    }, [onBlur, handleValidation]);
    
    // Focus handler
    const handleFocus = useCallback(() => {
      setIsFocused(true);
    }, []);
    
    // Memoize styles
    const containerStyles = useMemo<React.CSSProperties>(() => ({
      display: 'flex',
      flexDirection: 'column',
      width: fullWidth ? '100%' : 'auto',
      gap: theme.tokens.spacing.scale['1'],
    }), [fullWidth, theme]);
    
    const labelStyles = useMemo<React.CSSProperties>(() => ({
      fontSize: theme.tokens.typography.fontSize.sm,
      fontWeight: theme.tokens.typography.fontWeight.medium,
      color: hasError 
        ? theme.tokens.color.error 
        : theme.tokens.color.text.primary,
      marginBottom: theme.tokens.spacing.scale['1'],
    }), [theme, hasError]);
    
    const inputWrapperStyles = useMemo<React.CSSProperties>(() => ({
      display: 'flex',
      alignItems: 'center',
      position: 'relative',
      gap: theme.tokens.spacing.scale['2'],
    }), [theme]);
    
    const sizeMap = useMemo(() => ({
      sm: {
        height: '32px',
        padding: `0 ${theme.tokens.spacing.scale['3']}`,
        fontSize: theme.tokens.typography.fontSize.sm,
      },
      md: {
        height: '40px',
        padding: `0 ${theme.tokens.spacing.scale['4']}`,
        fontSize: theme.tokens.typography.fontSize.base,
      },
      lg: {
        height: '48px',
        padding: `0 ${theme.tokens.spacing.scale['5']}`,
        fontSize: theme.tokens.typography.fontSize.lg,
      },
    }), [theme]);
    
    const variantStyles = useMemo(() => ({
      outlined: {
        background: 'transparent',
        border: `1px solid ${hasError 
          ? theme.tokens.color.error 
          : isFocused 
            ? theme.tokens.color.primary 
            : theme.tokens.color.border}`,
        boxShadow: isFocused ? `0 0 0 3px ${theme.tokens.color.primary}22` : 'none',
      },
      filled: {
        background: theme.tokens.color.backgroundSecondary || theme.tokens.color.surface,
        border: `1px solid ${hasError 
          ? theme.tokens.color.error 
          : 'transparent'}`,
        boxShadow: isFocused ? `0 0 0 3px ${theme.tokens.color.primary}22` : 'none',
      },
      unstyled: {
        background: 'transparent',
        border: 'none',
        boxShadow: 'none',
      },
    }), [theme, hasError, isFocused]);
    
    const inputStyles = useMemo<React.CSSProperties>(() => ({
      ...sizeMap[size],
      ...variantStyles[variant],
      width: '100%',
      fontFamily: theme.tokens.typography.fontFamily.base,
      color: theme.tokens.color.text.primary,
      borderRadius: theme.tokens.shape.borderRadius.md,
      outline: 'none',
      transition: `all ${theme.tokens.animation.duration.fast} ${theme.tokens.animation.easing.easeInOut}`,
      opacity: disabled ? 0.6 : 1,
      cursor: disabled ? 'not-allowed' : 'text',
      paddingLeft: startIcon ? `${theme.tokens.spacing.scale['10']}` : sizeMap[size].padding,
      paddingRight: (endIcon || loading) ? `${theme.tokens.spacing.scale['10']}` : sizeMap[size].padding,
    }), [theme, size, variant, variantStyles, sizeMap, disabled, startIcon, endIcon, loading]);
    
    const iconStyles = useMemo<React.CSSProperties>(() => ({
      position: 'absolute',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      color: hasError 
        ? theme.tokens.color.error 
        : theme.tokens.color.text.secondary,
      pointerEvents: 'none',
    }), [theme, hasError]);
    
    const helperTextStyles = useMemo<React.CSSProperties>(() => ({
      fontSize: theme.tokens.typography.fontSize.xs,
      color: hasError 
        ? theme.tokens.color.error 
        : theme.tokens.color.text.secondary,
      marginTop: theme.tokens.spacing.scale['1'],
    }), [theme, hasError]);
    
    const inputId = props.id || `input-${Math.random().toString(36).substr(2, 9)}`;
    
    return (
      <div style={containerStyles} className={cn('sutra-input-container', className)}>
        {label && (
          <label htmlFor={inputId} style={labelStyles} className="sutra-input__label">
            {label}
            {required && <span style={{ color: theme.tokens.color.error }}> *</span>}
          </label>
        )}
        
        <div style={inputWrapperStyles} className="sutra-input__wrapper">
          {startIcon && (
            <span 
              style={{ ...iconStyles, left: theme.tokens.spacing.scale['3'] }} 
              className="sutra-input__start-icon"
            >
              {startIcon}
            </span>
          )}
          
          <input
            ref={ref}
            id={inputId}
            className={cn(
              'sutra-input',
              `sutra-input--${size}`,
              `sutra-input--${variant}`,
              hasError && 'sutra-input--error',
              isFocused && 'sutra-input--focused'
            )}
            style={inputStyles}
            disabled={disabled}
            required={required}
            aria-invalid={hasError}
            aria-describedby={
              error ? `${inputId}-error` : 
              helperText ? `${inputId}-helper` : 
              undefined
            }
            onChange={handleChange}
            onBlur={handleBlur}
            onFocus={handleFocus}
            {...props}
          />
          
          {loading && (
            <span 
              style={{ ...iconStyles, right: theme.tokens.spacing.scale['3'] }} 
              className="sutra-input__loader"
            >
              <Spinner size={size === 'sm' ? 14 : size === 'lg' ? 20 : 16} />
            </span>
          )}
          
          {!loading && endIcon && (
            <span 
              style={{ ...iconStyles, right: theme.tokens.spacing.scale['3'] }} 
              className="sutra-input__end-icon"
            >
              {endIcon}
            </span>
          )}
        </div>
        
        {(error || helperText) && (
          <span
            id={error ? `${inputId}-error` : `${inputId}-helper`}
            style={helperTextStyles}
            className={cn('sutra-input__helper-text', hasError && 'sutra-input__helper-text--error')}
            role={error ? 'alert' : undefined}
          >
            {error || helperText}
          </span>
        )}
      </div>
    );
  }
);

// Simple spinner component
function Spinner({ size }: { size: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      style={{
        animation: 'spin 1s linear infinite',
      }}
    >
      <circle
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="4"
        strokeOpacity="0.25"
      />
      <path
        d="M12 2a10 10 0 0 1 10 10"
        stroke="currentColor"
        strokeWidth="4"
        strokeLinecap="round"
      />
      <style>{`
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </svg>
  );
}

// Export memoized component
export const Input = memo(InputComponent);
Input.displayName = 'Input';
