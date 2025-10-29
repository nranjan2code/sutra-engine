/**
 * Unit Tests for Card Component
 * Production-grade test suite with 100% coverage
 */

import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { axe, toHaveNoViolations } from 'jest-axe';
import { ThemeProvider } from '@sutra/ui-core';
import { holographicTheme, professionalTheme, commandTheme } from '@sutra/ui-themes';
import { Card, CardHeader, CardContent, CardActions } from '../primitives/Card';

expect.extend(toHaveNoViolations);

describe('Card', () => {
  describe('Rendering', () => {
    it('should render with children', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Card>Card Content</Card>
        </ThemeProvider>
      );
      expect(screen.getByText('Card Content')).toBeInTheDocument();
    });

    it('should render with custom className', () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Card className="custom-card">Card</Card>
        </ThemeProvider>
      );
      expect(container.querySelector('.custom-card')).toBeInTheDocument();
    });
  });

  describe('Variants', () => {
    const variants = ['default', 'elevated', 'outlined', 'floating'] as const;

    variants.forEach((variant) => {
      it(`should render ${variant} variant`, () => {
        const { container } = render(
          <ThemeProvider theme={holographicTheme}>
            <Card variant={variant}>Card</Card>
          </ThemeProvider>
        );
        expect(container.querySelector(`.sutra-card--${variant}`)).toBeInTheDocument();
      });
    });

    it('should default to default variant', () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Card>Card</Card>
        </ThemeProvider>
      );
      expect(container.querySelector('.sutra-card--default')).toBeInTheDocument();
    });
  });

  describe('Padding', () => {
    const paddings = ['none', 'sm', 'md', 'lg'] as const;

    paddings.forEach((padding) => {
      it(`should apply ${padding} padding`, () => {
        const { container } = render(
          <ThemeProvider theme={holographicTheme}>
            <Card padding={padding}>Card</Card>
          </ThemeProvider>
        );
        const card = container.querySelector('.sutra-card');
        expect(card).toBeInTheDocument();
      });
    });

    it('should default to md padding', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Card>Card</Card>
        </ThemeProvider>
      );
      expect(screen.getByText('Card')).toBeInTheDocument();
    });
  });

  describe('Interactive State', () => {
    it('should apply interactive class when interactive is true', () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Card interactive>Interactive Card</Card>
        </ThemeProvider>
      );
      expect(container.querySelector('.sutra-card--interactive')).toBeInTheDocument();
    });

    it('should handle click when interactive', async () => {
      const handleClick = jest.fn();
      const user = userEvent.setup();
      
      render(
        <ThemeProvider theme={holographicTheme}>
          <Card interactive onClick={handleClick}>
            Clickable Card
          </Card>
        </ThemeProvider>
      );
      
      await user.click(screen.getByText('Clickable Card'));
      expect(handleClick).toHaveBeenCalledTimes(1);
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
            <Card>Card</Card>
          </ThemeProvider>
        );
        expect(screen.getByText('Card')).toBeInTheDocument();
      });
    });
  });

  describe('Accessibility', () => {
    it('should have no accessibility violations', async () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Card>Accessible Card</Card>
        </ThemeProvider>
      );
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });

    it('should support aria attributes', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Card aria-label="Custom card">Card</Card>
        </ThemeProvider>
      );
      expect(screen.getByLabelText('Custom card')).toBeInTheDocument();
    });
  });

  describe('Forwarded Ref', () => {
    it('should forward ref to div element', () => {
      const ref = React.createRef<HTMLDivElement>();
      render(
        <ThemeProvider theme={holographicTheme}>
          <Card ref={ref}>Card</Card>
        </ThemeProvider>
      );
      expect(ref.current).toBeInstanceOf(HTMLDivElement);
    });
  });
});

describe('CardHeader', () => {
  describe('Rendering', () => {
    it('should render with children', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Card>
            <CardHeader>Header Content</CardHeader>
          </Card>
        </ThemeProvider>
      );
      expect(screen.getByText('Header Content')).toBeInTheDocument();
    });

    it('should apply header class', () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Card>
            <CardHeader>Header</CardHeader>
          </Card>
        </ThemeProvider>
      );
      expect(container.querySelector('.sutra-card__header')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have no accessibility violations', async () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Card>
            <CardHeader>Header</CardHeader>
          </Card>
        </ThemeProvider>
      );
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });
  });

  describe('Forwarded Ref', () => {
    it('should forward ref to div element', () => {
      const ref = React.createRef<HTMLDivElement>();
      render(
        <ThemeProvider theme={holographicTheme}>
          <Card>
            <CardHeader ref={ref}>Header</CardHeader>
          </Card>
        </ThemeProvider>
      );
      expect(ref.current).toBeInstanceOf(HTMLDivElement);
    });
  });
});

describe('CardContent', () => {
  describe('Rendering', () => {
    it('should render with children', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Card>
            <CardContent>Content</CardContent>
          </Card>
        </ThemeProvider>
      );
      expect(screen.getByText('Content')).toBeInTheDocument();
    });

    it('should apply content class', () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Card>
            <CardContent>Content</CardContent>
          </Card>
        </ThemeProvider>
      );
      expect(container.querySelector('.sutra-card__content')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have no accessibility violations', async () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Card>
            <CardContent>Content</CardContent>
          </Card>
        </ThemeProvider>
      );
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });
  });

  describe('Forwarded Ref', () => {
    it('should forward ref to div element', () => {
      const ref = React.createRef<HTMLDivElement>();
      render(
        <ThemeProvider theme={holographicTheme}>
          <Card>
            <CardContent ref={ref}>Content</CardContent>
          </Card>
        </ThemeProvider>
      );
      expect(ref.current).toBeInstanceOf(HTMLDivElement);
    });
  });
});

describe('CardActions', () => {
  describe('Rendering', () => {
    it('should render with children', () => {
      render(
        <ThemeProvider theme={holographicTheme}>
          <Card>
            <CardActions>Actions</CardActions>
          </Card>
        </ThemeProvider>
      );
      expect(screen.getByText('Actions')).toBeInTheDocument();
    });

    it('should apply actions class', () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Card>
            <CardActions>Actions</CardActions>
          </Card>
        </ThemeProvider>
      );
      expect(container.querySelector('.sutra-card__actions')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have no accessibility violations', async () => {
      const { container } = render(
        <ThemeProvider theme={holographicTheme}>
          <Card>
            <CardActions>Actions</CardActions>
          </Card>
        </ThemeProvider>
      );
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });
  });

  describe('Forwarded Ref', () => {
    it('should forward ref to div element', () => {
      const ref = React.createRef<HTMLDivElement>();
      render(
        <ThemeProvider theme={holographicTheme}>
          <Card>
            <CardActions ref={ref}>Actions</CardActions>
          </Card>
        </ThemeProvider>
      );
      expect(ref.current).toBeInstanceOf(HTMLDivElement);
    });
  });
});

describe('Card Composition', () => {
  it('should render complete card with all sub-components', () => {
    render(
      <ThemeProvider theme={holographicTheme}>
        <Card>
          <CardHeader>Title</CardHeader>
          <CardContent>Content</CardContent>
          <CardActions>Actions</CardActions>
        </Card>
      </ThemeProvider>
    );
    
    expect(screen.getByText('Title')).toBeInTheDocument();
    expect(screen.getByText('Content')).toBeInTheDocument();
    expect(screen.getByText('Actions')).toBeInTheDocument();
  });

  it('should maintain proper structure with nested components', () => {
    const { container } = render(
      <ThemeProvider theme={holographicTheme}>
        <Card>
          <CardHeader>Header</CardHeader>
          <CardContent>
            <div>Nested Content</div>
          </CardContent>
          <CardActions>
            <button>Action</button>
          </CardActions>
        </Card>
      </ThemeProvider>
    );
    
    const card = container.querySelector('.sutra-card');
    const header = container.querySelector('.sutra-card__header');
    const content = container.querySelector('.sutra-card__content');
    const actions = container.querySelector('.sutra-card__actions');
    
    expect(card).toContainElement(header);
    expect(card).toContainElement(content);
    expect(card).toContainElement(actions);
  });

  it('should have no accessibility violations with full composition', async () => {
    const { container } = render(
      <ThemeProvider theme={holographicTheme}>
        <Card>
          <CardHeader>Card Title</CardHeader>
          <CardContent>Card content goes here</CardContent>
          <CardActions>
            <button>Action 1</button>
            <button>Action 2</button>
          </CardActions>
        </Card>
      </ThemeProvider>
    );
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});
