# Sutra UI Core

Core foundation for the Sutra UI Framework - theme system, hooks, and utilities.

## Installation

```bash
npm install @sutra/ui-core
# or
pnpm add @sutra/ui-core
```

## Features

- 🎨 **Theme System** - Powerful theming with React context
- 🪝 **Utility Hooks** - Responsive design, accessibility, and more
- 🛠️ **Helper Functions** - Color manipulation, layout utilities
- 📱 **Responsive** - Built-in breakpoint system
- ♿ **Accessible** - WCAG compliance helpers
- 🔒 **Type-Safe** - Full TypeScript support

## Quick Start

```typescript
import { ThemeProvider, createTheme } from '@sutra/ui-core';

const theme = createTheme({
  name: 'my-theme',
  displayName: 'My Theme',
  tokens: {
    color: {
      primary: '#6366f1',
      background: '#ffffff',
    },
  },
});

function App() {
  return (
    <ThemeProvider theme={theme}>
      <YourApp />
    </ThemeProvider>
  );
}
```

## Usage

### Using Theme in Components

```typescript
import { useTheme } from '@sutra/ui-core';

function MyComponent() {
  const { theme } = useTheme();
  
  return (
    <div style={{ 
      backgroundColor: theme.tokens.color.surface,
      color: theme.tokens.color.text.primary 
    }}>
      Themed content
    </div>
  );
}
```

### Responsive Design

```typescript
import { useBreakpoint, useResponsiveValue } from '@sutra/ui-core';

function ResponsiveComponent() {
  const breakpoint = useBreakpoint();
  const columns = useResponsiveValue({ xs: 1, sm: 2, md: 3, lg: 4 });
  
  return <div>Current breakpoint: {breakpoint}, Columns: {columns}</div>;
}
```

### Accessibility

```typescript
import { useReducedMotion, useFocusVisible } from '@sutra/ui-core';

function AccessibleComponent() {
  const prefersReducedMotion = useReducedMotion();
  const focusVisible = useFocusVisible();
  
  const animationDuration = prefersReducedMotion ? '0ms' : '250ms';
  
  return <div>Accessible and respectful of user preferences</div>;
}
```

### Color Utilities

```typescript
import { hexToRgba, lighten, darken, contrastRatio } from '@sutra/ui-core';

const transparentBlue = hexToRgba('#0000ff', 0.5); // rgba(0, 0, 255, 0.5)
const lighterBlue = lighten('#0000ff', 0.2);
const ratio = contrastRatio('#000000', '#ffffff'); // 21
```

## API Reference

See [documentation](../../docs/ui-framework/) for complete API reference.

## License

MIT
