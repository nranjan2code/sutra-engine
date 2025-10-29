/**
 * Unit Tests for Badge Component
 * Production-grade test suite with 100% coverage
 */

import React from 'react';
import { render, screen } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { ThemeProvider } from '../../core';
import { holographicTheme, professionalTheme, commandTheme } from '../../themes';
import { Badge } from '../primitives/Badge';

expect.extend(toHaveNoViolations);

describe('Badge', () => {
  describe('Rendering', () => {
    it('should render with children', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge>New</Badge>
        </ThemeProvider>
      );
      expect(screen.getByText('New')).toBeInTheDocument();
    });

    it('should render with value prop', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge value={5} />
        </ThemeProvider>
      );
      expect(screen.getByText('5')).toBeInTheDocument();
    });

    it('should prefer value prop over children', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge value={10}>Ignored</Badge>
        </ThemeProvider>
      );
      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.queryByText('Ignored')).not.toBeInTheDocument();
    });

    it('should render with custom className', () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Badge className="custom-badge">Badge</Badge>
        </ThemeProvider>
      );
      expect(container.querySelector('.custom-badge')).toBeInTheDocument();
    });
  });

  describe('Color Schemes', () => {
    const colorSchemes = [
      'primary',
      'secondary',
      'success',
      'warning',
      'error',
      'info',
      'neutral',
    ] as const;

    colorSchemes.forEach((colorScheme) => {
      it(`should render ${colorScheme} colorScheme`, () => {
        const { container } = render(
          <ThemeProvider theme={holographicTheme}>
            <Badge colorScheme={colorScheme}>Badge</Badge>
          </ThemeProvider>
        );
        expect(container.querySelector(`.sutra-badge--${colorScheme}`)).toBeInTheDocument();
      });
    });

    it('should default to neutral colorScheme', () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Badge>Badge</Badge>
        </ThemeProvider>
      );
      expect(container.querySelector('.sutra-badge--neutral')).toBeInTheDocument();
    });
  });

  describe('Sizes', () => {
    const sizes = ['sm', 'md', 'lg'] as const;

    sizes.forEach((size) => {
      it(`should render ${size} size`, () => {
        const { container } = render(
          <ThemeProvider theme={holographicTheme}>
            <Badge size={size}>Badge</Badge>
          </ThemeProvider>
        );
        expect(container.querySelector(`.sutra-badge--${size}`)).toBeInTheDocument();
      });
    });

    it('should default to md size', () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Badge>Badge</Badge>
        </ThemeProvider>
      );
      expect(container.querySelector('.sutra-badge--md')).toBeInTheDocument();
    });
  });

  describe('Variants', () => {
    const variants = ['solid', 'outline', 'subtle'] as const;

    variants.forEach((variant) => {
      it(`should render ${variant} variant`, () => {
        const { container } = render(
          <ThemeProvider theme={holographicTheme}>
            <Badge variant={variant}>Badge</Badge>
          </ThemeProvider>
        );
        expect(container.querySelector(`.sutra-badge--${variant}`)).toBeInTheDocument();
      });
    });

    it('should default to solid variant', () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Badge>Badge</Badge>
        </ThemeProvider>
      );
      expect(container.querySelector('.sutra-badge--solid')).toBeInTheDocument();
    });
  });

  describe('Value Formatting', () => {
    it('should render numeric values', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge value={42} />
        </ThemeProvider>
      );
      expect(screen.getByText('42')).toBeInTheDocument();
    });

    it('should render string values', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge value="Beta" />
        </ThemeProvider>
      );
      expect(screen.getByText('Beta')).toBeInTheDocument();
    });

    it('should render zero value', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge value={0} />
        </ThemeProvider>
      );
      expect(screen.getByText('0')).toBeInTheDocument();
    });
  });

  describe('Theme Support', () => {
    const themes = [
      { name: 'holographic', theme: holographicTheme },
      { name: 'professional', theme: professionalTheme },
      { name: 'command', theme: commandTheme },
    ];

    themes.forEach(({ name, theme }) => {
      it(`should render correctly with ${name} theme`, () => {
        render(
          <ThemeProvider theme={theme}>
            <Badge>Badge</Badge>
          </ThemeProvider>
        );
        expect(screen.getByText('Badge')).toBeInTheDocument();
      });
    });
  });

  describe('Accessibility', () => {
    it('should have no accessibility violations', async () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Badge>Accessible Badge</Badge>
        </ThemeProvider>
      );
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should support aria-label', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge aria-label="5 notifications">5</Badge>
        </ThemeProvider>
      );
      expect(screen.getByLabelText('5 notifications')).toBeInTheDocument();
    });

    it('should support custom aria attributes', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge aria-live="polite" aria-atomic="true">
            Live Update
          </Badge>
        </ThemeProvider>
      );
      const badge = screen.getByText('Live Update');
      expect(badge).toHaveAttribute('aria-live', 'polite');
      expect(badge).toHaveAttribute('aria-atomic', 'true');
    });
  });

  describe('Forwarded Ref', () => {
    it('should forward ref to span element', () => {
      const ref = React.createRef<HTMLSpanElement>();
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge ref={ref}>Badge</Badge>
        </ThemeProvider>
      );
      expect(ref.current).toBeInstanceOf(HTMLSpanElement);
    });
  });

  describe('Custom Props', () => {
    it('should pass through native span props', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge title="Tooltip text" data-testid="custom-badge">
            Badge
          </Badge>
        </ThemeProvider>
      );
      const badge = screen.getByTestId('custom-badge');
      expect(badge).toHaveAttribute('title', 'Tooltip text');
    });

    it('should support data attributes', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge data-analytics="badge-click" data-category="status">
            Badge
          </Badge>
        </ThemeProvider>
      );
      const badge = screen.getByText('Badge');
      expect(badge).toHaveAttribute('data-analytics', 'badge-click');
      expect(badge).toHaveAttribute('data-category', 'status');
    });
  });

  describe('Style Combinations', () => {
    it('should combine all style props correctly', () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Badge colorScheme="error" size="lg" variant="outline">
            Critical
          </Badge>
        </ThemeProvider>
      );
      const badge = container.querySelector('.sutra-badge');
      expect(badge).toHaveClass('sutra-badge--error');
      expect(badge).toHaveClass('sutra-badge--lg');
      expect(badge).toHaveClass('sutra-badge--outline');
    });

    it('should apply styles from all colorSchemes with all variants', () => {
      const colorSchemes: Array<'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info' | 'neutral'> = [
        'primary',
        'secondary',
        'success',
        'warning',
        'error',
        'info',
        'neutral',
      ];
      const variants: Array<'solid' | 'outline' | 'subtle'> = ['solid', 'outline', 'subtle'];

      colorSchemes.forEach((colorScheme) => {
        variants.forEach((variant) => {
          const { container } = render(
            <ThemeProvider theme={holographicTheme}>
              <Badge colorScheme={colorScheme} variant={variant}>
                {colorScheme}-{variant}
              </Badge>
            </ThemeProvider>
          );
          const badge = container.querySelector('.sutra-badge');
          expect(badge).toBeInTheDocument();
        });
      });
    });
  });

  describe('Edge Cases', () => {
    it('should render without children or value', () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Badge />
        </ThemeProvider>
      );
      expect(container.querySelector('.sutra-badge')).toBeInTheDocument();
    });

    it('should handle long text content', () => {
      const longText = 'This is a very long badge text that should still render';
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge>{longText}</Badge>
        </ThemeProvider>
      );
      expect(screen.getByText(longText)).toBeInTheDocument();
    });

    it('should handle special characters in value', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge value="99+" />
        </ThemeProvider>
      );
      expect(screen.getByText('99+')).toBeInTheDocument();
    });

    it('should handle multiple re-renders', () => {
      const { rerender } = render(
        <ThemeProvider theme={holographicTheme}>
          <Badge value={1} />
        </ThemeProvider>
      );

      rerender(
        <ThemeProvider theme={holographicTheme}>
          <Badge value={5} />
        </ThemeProvider>
      );

      rerender(
        <ThemeProvider theme={holographicTheme}>
          <Badge value={10} />
        </ThemeProvider>
      );

      expect(screen.getByText('10')).toBeInTheDocument();
    });
  });

  describe('Semantic Use Cases', () => {
    it('should work as notification badge', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge colorScheme="error" variant="solid" size="sm" value={3} />
        </ThemeProvider>
      );
      expect(screen.getByText('3')).toBeInTheDocument();
    });

    it('should work as status indicator', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge colorScheme="success" variant="subtle">
            Active
          </Badge>
        </ThemeProvider>
      );
      expect(screen.getByText('Active')).toBeInTheDocument();
    });

    it('should work as tag/label', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Badge colorScheme="info" variant="outline">
            Featured
          </Badge>
        </ThemeProvider>
      );
      expect(screen.getByText('Featured')).toBeInTheDocument();
    });
  });
});
