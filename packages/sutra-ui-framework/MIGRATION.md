# UI Framework Consolidation - Migration Guide

## What Changed?

The Sutra UI system has been consolidated from three separate packages into a single unified package:

### Before (Split Packages)
```
packages/
├── sutra-ui-core/          # Theme system, hooks, utilities
├── sutra-ui-themes/        # Theme definitions (holographic, professional, command)
└── sutra-ui-components/    # React components (Button, Card, Badge, etc.)
```

### After (Unified Package)
```
packages/
└── sutra-ui-framework/
    └── src/
        ├── core/           # Theme system, hooks, utilities
        ├── themes/         # Theme definitions
        └── components/     # React components
```

## Why Consolidate?

1. **Simpler Installation**: One package instead of three
2. **Better Developer Experience**: All UI code in one place
3. **Easier Maintenance**: Single build, test, and release process
4. **Clearer Architecture**: Framework vs. separate packages
5. **Reduced Complexity**: No inter-package dependencies to manage

## Migration for Users

### Old Import Pattern
```typescript
import { ThemeProvider } from '@sutra/ui-core';
import { holographicTheme } from '@sutra/ui-themes';
import { Button, Card, Badge } from '@sutra/ui-components';
```

### New Import Pattern (Unified)
```typescript
import { ThemeProvider, holographicTheme, Button, Card, Badge } from '@sutra/ui-framework';
```

### Installation

**Old:**
```bash
pnpm add @sutra/ui-core @sutra/ui-themes @sutra/ui-components
```

**New:**
```bash
pnpm add @sutra/ui-framework
```

## Package Exports

The unified package provides flexible exports:

```typescript
// Import everything from main export
import { ThemeProvider, holographicTheme, Button } from '@sutra/ui-framework';

// Or import from specific modules
import { ThemeProvider } from '@sutra/ui-framework/core';
import { holographicTheme } from '@sutra/ui-framework/themes';
import { Button } from '@sutra/ui-framework/components';
```

## Directory Structure

```
sutra-ui-framework/
├── src/
│   ├── index.ts                   # Main exports
│   ├── core/                      # Foundation layer
│   │   ├── context/               # ThemeProvider, ThemeContext
│   │   ├── hooks/                 # useTheme, useMediaQuery
│   │   ├── types/                 # Theme, ThemeTokens, ColorMode
│   │   └── utils/                 # cn (classnames utility)
│   ├── themes/                    # Theme definitions
│   │   ├── holographic/           # Cyan futuristic theme
│   │   ├── professional/          # Purple business theme
│   │   └── command/               # Indigo dark theme
│   └── components/                # React components
│       ├── Button/
│       ├── Card/
│       ├── Badge/
│       ├── Text/
│       └── Input/
├── .storybook/                    # Storybook configuration
├── jest.config.js                 # Jest testing config
├── jest.setup.ts                  # Jest setup file
├── tsconfig.json                  # TypeScript config
├── package.json                   # Package definition
├── README.md                      # Main documentation
├── QUICK_START.md                 # Quick start guide
├── IMPLEMENTATION_SUMMARY.md      # Implementation details
└── MIGRATION.md                   # This file
```

## Breaking Changes

**None!** The API remains exactly the same. Only the import paths have changed.

## Development Workflow

### Building
```bash
cd packages/sutra-ui-framework
pnpm install
pnpm build
```

### Testing
```bash
pnpm test              # Run all tests with coverage
pnpm test:watch        # Watch mode for TDD
pnpm test:ci           # CI optimized
```

### Storybook
```bash
pnpm storybook         # Start dev server
pnpm build-storybook   # Build static site
```

## For Contributors

### File Organization

All code is organized by layer:

1. **Core** (`src/core/`): Foundation code used by themes and components
2. **Themes** (`src/themes/`): Theme definitions that use core types
3. **Components** (`src/components/`): UI components that use core and themes

### Adding New Components

```typescript
// src/components/NewComponent/NewComponent.tsx
import { useTheme } from '../../core';
import { cn } from '../../core/utils';

export const NewComponent = () => {
  const theme = useTheme();
  // Component implementation
};
```

### Adding New Themes

```typescript
// src/themes/mytheme/index.ts
import type { Theme } from '../../core/types';

export const myTheme: Theme = {
  name: 'mytheme',
  colors: { /* ... */ },
  typography: { /* ... */ },
  spacing: { /* ... */ },
};
```

## Timeline

- **October 29, 2025**: Consolidation completed
- Old packages (`sutra-ui-core`, `sutra-ui-themes`, `sutra-ui-components`) removed
- New unified package (`sutra-ui-framework`) is the single source of truth

## Support

For questions or issues with the migration:

1. Check this migration guide
2. Review the [QUICK_START.md](./QUICK_START.md) for updated examples
3. See [README.md](./README.md) for API documentation
4. Explore Storybook for interactive examples: `pnpm storybook`

## Next Steps

1. Update your imports to use `@sutra/ui-framework`
2. Remove old packages from dependencies
3. Run your tests to verify everything works
4. Enjoy the simplified developer experience!
