# Sutra UI Framework

Complete UI framework for Sutra AI - unified package with core foundation, themes, and components. Built with TypeScript, theme-aware, and fully accessible.

## Installation

```bash
npm install @sutra/ui-framework
# or
pnpm add @sutra/ui-framework
```

## Quick Start

```typescript
import { ThemeProvider, holographicTheme, Button, Card, Badge, Text } from '@sutra/ui-framework';

function App() {
  return (
    <ThemeProvider theme={holographicTheme}>
      <Card>
        <Card.Header>
          <Text variant="h5">Welcome</Text>
          <Badge value="New" colorScheme="primary" />
        </Card.Header>
        <Card.Content>
          <Text variant="body2" color="secondary">
            Professional UI components with theme support
          </Text>
        </Card.Content>
        <Card.Actions>
          <Button variant="primary">Get Started</Button>
          <Button variant="ghost">Learn More</Button>
        </Card.Actions>
      </Card>
    </ThemeProvider>
  );
}
```

## Components

### Button

Interactive button with multiple variants and states.

```typescript
<Button variant="primary" size="md" onClick={() => console.log('clicked')}>
  Click Me
</Button>

<Button variant="secondary" loading>
  Loading...
</Button>

<Button variant="danger" icon={<Icon />}>
  Delete
</Button>
```

**Props:**
- `variant`: 'primary' | 'secondary' | 'ghost' | 'danger'
- `size`: 'sm' | 'md' | 'lg'
- `loading`: boolean
- `fullWidth`: boolean
- `icon` / `iconRight`: React.ReactNode

### Card

Versatile container with composition pattern.

```typescript
<Card variant="elevated" padding="lg">
  <Card.Header>
    <Text variant="h6">Title</Text>
  </Card.Header>
  <Card.Content>
    Content goes here
  </Card.Content>
  <Card.Actions>
    <Button>Action</Button>
  </Card.Actions>
</Card>
```

**Props:**
- `variant`: 'default' | 'elevated' | 'outlined' | 'floating'
- `padding`: 'none' | 'sm' | 'md' | 'lg'
- `interactive`: boolean

### Badge

Status indicator and label.

```typescript
<Badge colorScheme="success" size="sm">
  Active
</Badge>

<Badge variant="outline" colorScheme="warning">
  Pending
</Badge>

<Badge value={99} colorScheme="error" />
```

**Props:**
- `colorScheme`: 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info' | 'neutral'
- `size`: 'sm' | 'md' | 'lg'
- `variant`: 'solid' | 'outline' | 'subtle'
- `value`: number | string

### Text

Typography with semantic variants.

```typescript
<Text variant="h1">Heading 1</Text>
<Text variant="body1" color="secondary">
  Regular text with secondary color
</Text>
<Text variant="caption" weight="bold">
  Small bold caption
</Text>
```

**Props:**
- `variant`: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6' | 'body1' | 'body2' | 'caption' | 'overline'
- `color`: 'primary' | 'secondary' | 'tertiary' | 'disabled' | 'inherit'
- `weight`: 'light' | 'normal' | 'medium' | 'semibold' | 'bold'
- `align`: 'left' | 'center' | 'right'
- `as`: HTML element to render

## Theme Integration

All components automatically adapt to the active theme:

```typescript
import { holographicTheme, professionalTheme, commandTheme } from '@sutra/ui-framework';

// Same components, different themes
<ThemeProvider theme={holographicTheme}>
  <Button variant="primary">Holographic Style</Button>
</ThemeProvider>

<ThemeProvider theme={professionalTheme}>
  <Button variant="primary">Professional Style</Button>
</ThemeProvider>

<ThemeProvider theme={commandTheme}>
  <Button variant="primary">Command Style</Button>
</ThemeProvider>
```

## Accessibility

All components are built with accessibility in mind:
- Semantic HTML
- ARIA attributes
- Keyboard navigation
- Focus management
- Screen reader support
- WCAG AA+ compliant

## TypeScript Support

Full TypeScript definitions included:

```typescript
import type { ButtonProps, CardProps, BadgeProps, TextProps } from '@sutra/ui-framework';

const MyButton: React.FC<ButtonProps> = (props) => {
  return <Button {...props} />;
};
```

## License

MIT
