# Sutra UI Themes

Theme definitions for the Sutra UI Framework. Includes three professionally designed themes:

- **Holographic** - Futuristic sci-fi HUD for sutra-explorer
- **Professional** - Clean Material Design 3 for sutra-client  
- **Command** - Dark command center for sutra-control

## Installation

```bash
npm install @sutra/ui-themes @sutra/ui-core
# or
pnpm add @sutra/ui-themes @sutra/ui-core
```

## Usage

### Using a Theme

```typescript
import { ThemeProvider } from '@sutra/ui-core';
import { holographicTheme } from '@sutra/ui-themes';

function App() {
  return (
    <ThemeProvider theme={holographicTheme}>
      <YourApp />
    </ThemeProvider>
  );
}
```

### Switching Themes

```typescript
import { ThemeProvider } from '@sutra/ui-core';
import { holographicTheme, professionalTheme, commandTheme } from '@sutra/ui-themes';

const availableThemes = [holographicTheme, professionalTheme, commandTheme];

function App() {
  return (
    <ThemeProvider 
      theme={holographicTheme} 
      availableThemes={availableThemes}
      persist={true}
    >
      <YourApp />
    </ThemeProvider>
  );
}

// In a component:
function ThemeSwitcher() {
  const { theme, setTheme, availableThemes } = useTheme();
  
  return (
    <select value={theme.name} onChange={(e) => setTheme(e.target.value)}>
      {availableThemes?.map(t => (
        <option key={t.name} value={t.name}>{t.displayName}</option>
      ))}
    </select>
  );
}
```

## Theme Comparison

| Feature | Holographic | Professional | Command |
|---------|-------------|--------------|---------|
| **Primary Color** | Cyan (#00ffff) | Purple (#6750A4) | Indigo (#6366f1) |
| **Background** | Black (#000000) | Light (#FEF7FF) | Dark Blue-Gray (#0f1629) |
| **Aesthetic** | Sci-fi HUD | Material Design 3 | Command Center |
| **Glow Effects** | Strong | None | Subtle |
| **Contrast** | WCAG AAA (14.6:1) | WCAG AA (7.0:1) | WCAG AA+ (12.0:1) |
| **Best For** | Graph visualization | Business apps | Dashboards |

## Customizing Themes

```typescript
import { extendTheme } from '@sutra/ui-core';
import { holographicTheme } from '@sutra/ui-themes';

const customTheme = extendTheme(holographicTheme, {
  tokens: {
    color: {
      primary: '#ff00ff', // Change to magenta
    },
  },
});
```

## License

MIT
