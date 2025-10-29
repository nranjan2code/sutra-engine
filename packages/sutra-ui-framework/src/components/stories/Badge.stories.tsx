/**
 * Badge Component Stories
 * Comprehensive showcase of all color schemes, variants, and sizes
 */

import type { Meta, StoryObj } from '@storybook/react';
import { Badge } from '../primitives/Badge';
import type { BadgeProps } from '../primitives/Badge';

const meta = {
  title: 'Components/Badge',
  component: Badge as any,
  tags: ['autodocs'],
  argTypes: {
    colorScheme: {
      control: 'select',
      options: ['primary', 'secondary', 'success', 'warning', 'error', 'info', 'neutral'],
      description: 'Color scheme for the badge',
    },
    variant: {
      control: 'select',
      options: ['solid', 'outline', 'subtle'],
      description: 'Visual style variant',
    },
    size: {
      control: 'select',
      options: ['sm', 'md', 'lg'],
      description: 'Badge size',
    },
    value: {
      control: 'text',
      description: 'Badge content (alternative to children)',
    },
  },
  args: {
    colorScheme: 'primary' as const,
    variant: 'solid' as const,
    size: 'md' as const,
  },
} satisfies Meta<BadgeProps>;

export default meta;
type Story = StoryObj<BadgeProps>;

// Basic Examples
export const Default: Story = {
  args: {
    children: 'Badge',
  },
};

export const WithValue: Story = {
  args: {
    value: 'New',
  },
};

// Color Schemes
export const AllColors: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
      <Badge colorScheme="primary">Primary</Badge>
      <Badge colorScheme="secondary">Secondary</Badge>
      <Badge colorScheme="success">Success</Badge>
      <Badge colorScheme="warning">Warning</Badge>
      <Badge colorScheme="error">Error</Badge>
      <Badge colorScheme="info">Info</Badge>
      <Badge colorScheme="neutral">Neutral</Badge>
    </div>
  ),
};

// Variants
export const SolidVariant: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
      <Badge variant="solid" colorScheme="primary">Primary</Badge>
      <Badge variant="solid" colorScheme="success">Success</Badge>
      <Badge variant="solid" colorScheme="warning">Warning</Badge>
      <Badge variant="solid" colorScheme="error">Error</Badge>
    </div>
  ),
};

export const OutlineVariant: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
      <Badge variant="outline" colorScheme="primary">Primary</Badge>
      <Badge variant="outline" colorScheme="success">Success</Badge>
      <Badge variant="outline" colorScheme="warning">Warning</Badge>
      <Badge variant="outline" colorScheme="error">Error</Badge>
    </div>
  ),
};

export const SubtleVariant: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
      <Badge variant="subtle" colorScheme="primary">Primary</Badge>
      <Badge variant="subtle" colorScheme="success">Success</Badge>
      <Badge variant="subtle" colorScheme="warning">Warning</Badge>
      <Badge variant="subtle" colorScheme="error">Error</Badge>
    </div>
  ),
};

// Sizes
export const AllSizes: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center', flexWrap: 'wrap' }}>
      <Badge size="sm" colorScheme="primary">Small</Badge>
      <Badge size="md" colorScheme="primary">Medium</Badge>
      <Badge size="lg" colorScheme="primary">Large</Badge>
    </div>
  ),
};

// Status Indicators
export const StatusBadges: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
      <Badge colorScheme="success" variant="solid">Active</Badge>
      <Badge colorScheme="warning" variant="solid">Pending</Badge>
      <Badge colorScheme="error" variant="solid">Inactive</Badge>
      <Badge colorScheme="info" variant="solid">Draft</Badge>
      <Badge colorScheme="neutral" variant="solid">Archived</Badge>
    </div>
  ),
};

// Notification Counts
export const NotificationBadges: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap', alignItems: 'center' }}>
      <div style={{ position: 'relative', display: 'inline-block' }}>
        <span style={{ fontSize: '1.5rem' }}>🔔</span>
        <Badge 
          colorScheme="error" 
          size="sm" 
          style={{ 
            position: 'absolute', 
            top: '-4px', 
            right: '-8px',
            minWidth: '18px',
            height: '18px',
            borderRadius: '9px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center'
          }}
        >
          3
        </Badge>
      </div>
      
      <div style={{ position: 'relative', display: 'inline-block' }}>
        <span style={{ fontSize: '1.5rem' }}>✉️</span>
        <Badge 
          colorScheme="primary" 
          size="sm"
          style={{ 
            position: 'absolute', 
            top: '-4px', 
            right: '-8px',
            minWidth: '18px',
            height: '18px',
            borderRadius: '9px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center'
          }}
        >
          12
        </Badge>
      </div>
      
      <div style={{ position: 'relative', display: 'inline-block' }}>
        <span style={{ fontSize: '1.5rem' }}>📱</span>
        <Badge 
          colorScheme="success" 
          size="sm"
          style={{ 
            position: 'absolute', 
            top: '-4px', 
            right: '-8px',
            minWidth: '18px',
            height: '18px',
            borderRadius: '9px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center'
          }}
        >
          99+
        </Badge>
      </div>
    </div>
  ),
};

// Tags/Categories
export const CategoryTags: Story = {
  render: () => (
    <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
      <Badge variant="subtle" colorScheme="primary" size="sm">React</Badge>
      <Badge variant="subtle" colorScheme="info" size="sm">TypeScript</Badge>
      <Badge variant="subtle" colorScheme="success" size="sm">Node.js</Badge>
      <Badge variant="subtle" colorScheme="warning" size="sm">JavaScript</Badge>
      <Badge variant="subtle" colorScheme="secondary" size="sm">CSS</Badge>
      <Badge variant="subtle" colorScheme="neutral" size="sm">HTML</Badge>
    </div>
  ),
};

// Priority Levels
export const PriorityBadges: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
        <Badge colorScheme="error" variant="solid" size="sm">High</Badge>
        <span>Critical bug fix required</span>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
        <Badge colorScheme="warning" variant="solid" size="sm">Medium</Badge>
        <span>Feature enhancement</span>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
        <Badge colorScheme="info" variant="solid" size="sm">Low</Badge>
        <span>Documentation update</span>
      </div>
    </div>
  ),
};

// All Combinations
export const AllCombinations: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      {(['primary', 'secondary', 'success', 'warning', 'error', 'info', 'neutral'] as const).map((color) => (
        <div key={color} style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
          <strong style={{ textTransform: 'capitalize' }}>{color}</strong>
          <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
            {(['solid', 'outline', 'subtle'] as const).map((variant) => (
              <Badge key={variant} colorScheme={color} variant={variant}>
                {variant}
              </Badge>
            ))}
          </div>
        </div>
      ))}
    </div>
  ),
};
