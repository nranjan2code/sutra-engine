/**
 * Button Component Stories
 * Comprehensive showcase of all variants, sizes, and states
 */

import type { StoryObj } from '@storybook/react';
import { Button, type ButtonProps } from '../primitives/Button';

const meta = {
  title: 'Components/Button',
  component: Button,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['primary', 'secondary', 'ghost', 'danger'],
      description: 'Visual style variant',
    },
    size: {
      control: 'select',
      options: ['sm', 'md', 'lg'],
      description: 'Button size',
    },
    loading: {
      control: 'boolean',
      description: 'Loading state with spinner',
    },
    disabled: {
      control: 'boolean',
      description: 'Disabled state',
    },
    fullWidth: {
      control: 'boolean',
      description: 'Full width button',
    },
    onClick: { action: 'clicked' },
  },
};

export default meta;
type Story = StoryObj<ButtonProps>;

// Default Button
export const Default: Story = {
  args: {
    children: 'Button',
    variant: 'primary',
    size: 'md',
  },
};

// Variants
export const Primary: Story = {
  args: {
    children: 'Primary Button',
    variant: 'primary',
  },
};

export const Secondary: Story = {
  args: {
    children: 'Secondary Button',
    variant: 'secondary',
  },
};

export const Ghost: Story = {
  args: {
    children: 'Ghost Button',
    variant: 'ghost',
  },
};

export const Danger: Story = {
  args: {
    children: 'Danger Button',
    variant: 'danger',
  },
};

// Sizes
export const Small: Story = {
  args: {
    children: 'Small Button',
    size: 'sm',
  },
};

export const Medium: Story = {
  args: {
    children: 'Medium Button',
    size: 'md',
  },
};

export const Large: Story = {
  args: {
    children: 'Large Button',
    size: 'lg',
  },
};

// States
export const Loading: Story = {
  args: {
    children: 'Loading Button',
    loading: true,
  },
};

export const Disabled: Story = {
  args: {
    children: 'Disabled Button',
    disabled: true,
  },
};

export const FullWidth: Story = {
  args: {
    children: 'Full Width Button',
    fullWidth: true,
  },
};

// With Icons
export const WithIcon: Story = {
  args: {
    children: 'Button with Icon',
    icon: <span>★</span>,
  },
};

export const WithIconRight: Story = {
  args: {
    children: 'Button with Right Icon',
    iconRight: <span>→</span>,
  },
};

export const WithBothIcons: Story = {
  args: {
    children: 'Both Icons',
    icon: <span>←</span>,
    iconRight: <span>→</span>,
  },
};

export const IconOnly: Story = {
  args: {
    icon: <span style={{ fontSize: '20px' }}>♥</span>,
    'aria-label': 'Like',
  },
};

// Interactive Examples
export const AllVariants: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap' }}>
      <Button variant="primary">Primary</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="danger">Danger</Button>
    </div>
  ),
};

export const AllSizes: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
      <Button size="sm">Small</Button>
      <Button size="md">Medium</Button>
      <Button size="lg">Large</Button>
    </div>
  ),
};

export const AllStates: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '1rem', flexDirection: 'column', maxWidth: '300px' }}>
      <Button>Normal</Button>
      <Button loading>Loading</Button>
      <Button disabled>Disabled</Button>
      <Button fullWidth>Full Width</Button>
    </div>
  ),
};

export const VariantGrid: Story = {
  render: () => (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '1rem' }}>
      <Button variant="primary" size="sm">Small Primary</Button>
      <Button variant="primary" size="md">Medium Primary</Button>
      <Button variant="primary" size="lg">Large Primary</Button>
      
      <Button variant="secondary" size="sm">Small Secondary</Button>
      <Button variant="secondary" size="md">Medium Secondary</Button>
      <Button variant="secondary" size="lg">Large Secondary</Button>
      
      <Button variant="ghost" size="sm">Small Ghost</Button>
      <Button variant="ghost" size="md">Medium Ghost</Button>
      <Button variant="ghost" size="lg">Large Ghost</Button>
      
      <Button variant="danger" size="sm">Small Danger</Button>
      <Button variant="danger" size="md">Medium Danger</Button>
      <Button variant="danger" size="lg">Large Danger</Button>
    </div>
  ),
};

// Accessibility Example
export const WithAccessibilityLabels: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '1rem' }}>
      <Button aria-label="Save document" icon={<span>💾</span>}>
        Save
      </Button>
      <Button aria-label="Delete item" variant="danger" icon={<span>🗑️</span>}>
        Delete
      </Button>
      <Button aria-label="Settings" icon={<span>⚙️</span>} />
    </div>
  ),
};

// Real-world Examples
export const FormActions: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end', padding: '1rem', borderTop: '1px solid rgba(255,255,255,0.1)' }}>
      <Button variant="ghost">Cancel</Button>
      <Button variant="primary">Save Changes</Button>
    </div>
  ),
};

export const ToolbarActions: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '0.5rem', padding: '0.5rem', background: 'rgba(255,255,255,0.05)', borderRadius: '8px' }}>
      <Button size="sm" variant="ghost" icon={<span>◀</span>} aria-label="Previous" />
      <Button size="sm" variant="ghost" icon={<span>▶</span>} aria-label="Next" />
      <Button size="sm" variant="ghost" icon={<span>↻</span>} aria-label="Refresh" />
      <div style={{ width: '1px', background: 'rgba(255,255,255,0.1)', margin: '0 0.25rem' }} />
      <Button size="sm" variant="ghost" icon={<span>⚙️</span>} aria-label="Settings" />
    </div>
  ),
};

export const ActionGroup: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', maxWidth: '400px' }}>
      <h3 style={{ margin: '0 0 0.5rem 0', fontSize: '1.25rem' }}>Quick Actions</h3>
      <Button variant="primary" icon={<span>+</span>} fullWidth>
        Create New Project
      </Button>
      <Button variant="secondary" icon={<span>📁</span>} fullWidth>
        Open Existing Project
      </Button>
      <Button variant="ghost" icon={<span>📥</span>} fullWidth>
        Import from File
      </Button>
    </div>
  ),
};
