/**
 * Card Component Stories
 * Comprehensive showcase of all variants and compositions
 */

import type { Meta, StoryObj } from '@storybook/react';
import { Card, CardHeader, CardContent, CardActions } from '../primitives/Card';
import type { CardProps } from '../primitives/Card';
import { Button } from '../primitives/Button';
import { Text } from '../primitives/Text';
import { Badge } from '../primitives/Badge';

const meta = {
  title: 'Components/Card',
  component: Card as any,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['default', 'elevated', 'outlined', 'floating'],
      description: 'Visual style variant',
    },
    interactive: {
      control: 'boolean',
      description: 'Add hover effects and cursor pointer',
    },
  },
  args: {
    variant: 'default' as const,
    interactive: false,
  },
} satisfies Meta<CardProps>;

export default meta;
type Story = StoryObj<CardProps>;

// Basic Examples
export const Default: Story = {
  render: (args) => (
    <Card {...args}>
      <CardContent>
        <Text variant="body1">This is a default card with simple content.</Text>
      </CardContent>
    </Card>
  ),
};

export const WithHeader: Story = {
  render: (args) => (
    <Card {...args}>
      <CardHeader>
        <Text variant="h5">Card Title</Text>
      </CardHeader>
      <CardContent>
        <Text variant="body2" color="secondary">
          A card with a header section for titles and metadata.
        </Text>
      </CardContent>
    </Card>
  ),
};

export const WithActions: Story = {
  render: (args) => (
    <Card {...args}>
      <CardContent>
        <Text variant="body1">Card with action buttons at the bottom.</Text>
      </CardContent>
      <CardActions>
        <Button variant="ghost" size="sm">Cancel</Button>
        <Button variant="primary" size="sm">Save</Button>
      </CardActions>
    </Card>
  ),
};

export const Complete: Story = {
  render: (args) => (
    <Card {...args}>
      <CardHeader>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', width: '100%' }}>
          <Text variant="h5">Complete Card</Text>
          <Badge colorScheme="success" size="sm">Active</Badge>
        </div>
      </CardHeader>
      <CardContent>
        <Text variant="body1" style={{ marginBottom: '0.5rem' }}>
          This card demonstrates all sections working together.
        </Text>
        <Text variant="body2" color="secondary">
          Header, content, and actions create a complete user interface element.
        </Text>
      </CardContent>
      <CardActions>
        <Button variant="ghost" size="sm">Learn More</Button>
        <Button variant="primary" size="sm">Get Started</Button>
      </CardActions>
    </Card>
  ),
};

// Variants
export const Elevated: Story = {
  args: {
    variant: 'elevated' as const,
  },
  render: (args) => (
    <Card {...args}>
      <CardHeader>
        <Text variant="h6">Elevated Card</Text>
      </CardHeader>
      <CardContent>
        <Text variant="body2" color="secondary">
          Higher elevation creates stronger visual hierarchy.
        </Text>
      </CardContent>
    </Card>
  ),
};

export const Outlined: Story = {
  args: {
    variant: 'outlined' as const,
  },
  render: (args) => (
    <Card {...args}>
      <CardHeader>
        <Text variant="h6">Outlined Card</Text>
      </CardHeader>
      <CardContent>
        <Text variant="body2" color="secondary">
          Border instead of shadow for subtle separation.
        </Text>
      </CardContent>
    </Card>
  ),
};

export const Floating: Story = {
  args: {
    variant: 'floating' as const,
  },
  render: (args) => (
    <Card {...args}>
      <CardHeader>
        <Text variant="h6">Floating Card</Text>
      </CardHeader>
      <CardContent>
        <Text variant="body2" color="secondary">
          Maximum elevation for modal-like emphasis.
        </Text>
      </CardContent>
    </Card>
  ),
};

export const Interactive: Story = {
  args: {
    variant: 'elevated' as const,
    interactive: true,
  },
  render: (args) => (
    <Card {...args}>
      <CardHeader>
        <Text variant="h6">Interactive Card</Text>
      </CardHeader>
      <CardContent>
        <Text variant="body2" color="secondary">
          Hover over this card to see the interactive effect.
        </Text>
      </CardContent>
    </Card>
  ),
};

// Complex Examples
export const ProductCard: Story = {
  render: () => (
    <Card variant="elevated" style={{ maxWidth: '320px' }}>
      <CardHeader>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', width: '100%' }}>
          <div>
            <Text variant="h6">Premium Plan</Text>
            <Text variant="caption" color="secondary">Best value</Text>
          </div>
          <Badge colorScheme="primary" variant="solid">Popular</Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div style={{ marginBottom: '1rem' }}>
          <Text variant="h4" style={{ marginBottom: '0.25rem' }}>$29</Text>
          <Text variant="body2" color="secondary">per month</Text>
        </div>
        <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
          <li><Text variant="body2">✓ Unlimited projects</Text></li>
          <li><Text variant="body2">✓ Priority support</Text></li>
          <li><Text variant="body2">✓ Advanced analytics</Text></li>
          <li><Text variant="body2">✓ Custom integrations</Text></li>
        </ul>
      </CardContent>
      <CardActions>
        <Button variant="primary" fullWidth>Subscribe Now</Button>
      </CardActions>
    </Card>
  ),
};

export const ProfileCard: Story = {
  render: () => (
    <Card variant="elevated" interactive style={{ maxWidth: '280px' }}>
      <CardContent>
        <div style={{ textAlign: 'center', marginBottom: '1rem' }}>
          <div style={{ 
            width: '80px', 
            height: '80px', 
            borderRadius: '50%', 
            background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            margin: '0 auto 1rem',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '2rem',
            fontWeight: 'bold',
            color: 'white'
          }}>
            JD
          </div>
          <Text variant="h6">John Doe</Text>
          <Text variant="body2" color="secondary">Senior Developer</Text>
        </div>
        <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'center' }}>
          <Badge colorScheme="info" size="sm">React</Badge>
          <Badge colorScheme="success" size="sm">TypeScript</Badge>
          <Badge colorScheme="primary" size="sm">Node.js</Badge>
        </div>
      </CardContent>
      <CardActions>
        <Button variant="ghost" size="sm" fullWidth>View Profile</Button>
      </CardActions>
    </Card>
  ),
};

export const NotificationCard: Story = {
  render: () => (
    <Card variant="outlined" style={{ maxWidth: '400px' }}>
      <CardContent>
        <div style={{ display: 'flex', gap: '1rem', alignItems: 'start' }}>
          <div style={{
            width: '40px',
            height: '40px',
            borderRadius: '8px',
            background: 'linear-gradient(135deg, #00ffff 0%, #0080ff 100%)',
            flexShrink: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '1.25rem'
          }}>
            🎉
          </div>
          <div style={{ flex: 1 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '0.5rem' }}>
              <Text variant="body1" weight="semibold">New Achievement Unlocked!</Text>
              <Text variant="caption" color="secondary">2m ago</Text>
            </div>
            <Text variant="body2" color="secondary">
              You've completed 100 tasks. Keep up the great work!
            </Text>
          </div>
        </div>
      </CardContent>
    </Card>
  ),
};
