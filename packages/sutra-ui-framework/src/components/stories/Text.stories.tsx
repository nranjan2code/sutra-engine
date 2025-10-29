/**
 * Text Component Stories
 * Comprehensive showcase of all typography variants
 */

import type { Meta, StoryObj } from '@storybook/react';
import { Text } from '../primitives/Text';
import type { TextProps } from '../primitives/Text';

const meta = {
  title: 'Components/Text',
  component: Text as any,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'body1', 'body2', 'caption', 'overline'],
      description: 'Typography variant',
    },
    color: {
      control: 'select',
      options: ['primary', 'secondary', 'tertiary', 'disabled', 'inherit'],
      description: 'Text color from theme',
    },
    weight: {
      control: 'select',
      options: ['light', 'normal', 'medium', 'semibold', 'bold'],
      description: 'Font weight',
    },
    align: {
      control: 'select',
      options: ['left', 'center', 'right'],
      description: 'Text alignment',
    },
    as: {
      control: 'select',
      options: ['h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'p', 'span', 'div'],
      description: 'HTML element to render',
    },
  },
  args: {
    variant: 'body1' as const,
    color: 'primary' as const,
  },
} satisfies Meta<TextProps>;

export default meta;
type Story = StoryObj<TextProps>;

// Basic Example
export const Default: Story = {
  args: {
    children: 'The quick brown fox jumps over the lazy dog',
  },
};

// All Heading Variants
export const AllHeadings: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <Text variant="h1">Heading 1 - Display</Text>
      <Text variant="h2">Heading 2 - Page Title</Text>
      <Text variant="h3">Heading 3 - Section</Text>
      <Text variant="h4">Heading 4 - Subsection</Text>
      <Text variant="h5">Heading 5 - Component</Text>
      <Text variant="h6">Heading 6 - Small Heading</Text>
    </div>
  ),
};

// Body Text Variants
export const BodyText: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', maxWidth: '600px' }}>
      <Text variant="body1">
        Body 1 - Default body text. Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
        Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
      </Text>
      <Text variant="body2">
        Body 2 - Smaller body text. Ut enim ad minim veniam, quis nostrud exercitation ullamco 
        laboris nisi ut aliquip ex ea commodo consequat.
      </Text>
      <Text variant="caption">
        Caption - Small text for captions and helper text. Duis aute irure dolor in reprehenderit.
      </Text>
      <Text variant="overline">
        OVERLINE - ALL CAPS LABEL TEXT
      </Text>
    </div>
  ),
};

// Text Colors
export const AllColors: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <Text variant="body1" color="primary">Primary color text</Text>
      <Text variant="body1" color="secondary">Secondary color text</Text>
      <Text variant="body1" color="tertiary">Tertiary color text</Text>
      <Text variant="body1" color="disabled">Disabled color text</Text>
      <div style={{ background: '#333', padding: '1rem', borderRadius: '4px' }}>
        <Text variant="body1" color="inherit">Inherit color text</Text>
      </div>
    </div>
  ),
};

// Font Weights
export const AllWeights: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <Text variant="body1" weight="light">Light weight (300)</Text>
      <Text variant="body1" weight="normal">Normal weight (400)</Text>
      <Text variant="body1" weight="medium">Medium weight (500)</Text>
      <Text variant="body1" weight="semibold">Semibold weight (600)</Text>
      <Text variant="body1" weight="bold">Bold weight (700)</Text>
    </div>
  ),
};

// Text Alignment
export const AllAlignments: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', width: '100%', maxWidth: '600px' }}>
      <Text variant="body1" align="left">Left aligned text - default alignment</Text>
      <Text variant="body1" align="center">Center aligned text</Text>
      <Text variant="body1" align="right">Right aligned text</Text>
      <Text variant="body1" align="left">
        Left aligned paragraph text - Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
        Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
      </Text>
    </div>
  ),
};

// Semantic HTML Elements
export const SemanticElements: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <Text variant="h2" as="h1">H2 style as H1 element</Text>
      <Text variant="body1" as="p">Body text as paragraph</Text>
      <Text variant="caption" as="span">Caption as inline span</Text>
      <Text variant="h6" as="div">H6 style as div element</Text>
    </div>
  ),
};

// Article Example
export const ArticleExample: Story = {
  render: () => (
    <article style={{ maxWidth: '700px' }}>
      <Text variant="overline" color="secondary">Technology</Text>
      <Text variant="h2" style={{ marginTop: '0.5rem', marginBottom: '1rem' }}>
        The Future of Web Development
      </Text>
      <Text variant="caption" color="secondary" style={{ display: 'block', marginBottom: '2rem' }}>
        Published on October 29, 2025 • 5 min read
      </Text>
      
      <Text variant="body1" style={{ marginBottom: '1rem' }}>
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor 
        incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud 
        exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
      </Text>
      
      <Text variant="h4" style={{ marginTop: '2rem', marginBottom: '1rem' }}>
        Key Takeaways
      </Text>
      
      <Text variant="body2" color="secondary" style={{ marginBottom: '1rem' }}>
        Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu 
        fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in 
        culpa qui officia deserunt mollit anim id est laborum.
      </Text>
      
      <Text variant="caption" color="tertiary" style={{ display: 'block', marginTop: '2rem' }}>
        * This is a footnote or disclaimer text
      </Text>
    </article>
  ),
};

// Card Content Example
export const CardContentExample: Story = {
  render: () => (
    <div style={{ 
      maxWidth: '400px', 
      padding: '2rem', 
      border: '1px solid rgba(255,255,255,0.1)', 
      borderRadius: '8px' 
    }}>
      <Text variant="h5" style={{ marginBottom: '0.5rem' }}>
        Product Name
      </Text>
      <Text variant="body2" color="secondary" style={{ marginBottom: '1.5rem' }}>
        Short description of the product goes here
      </Text>
      <Text variant="h3" weight="bold" style={{ marginBottom: '0.25rem' }}>
        $99.99
      </Text>
      <Text variant="caption" color="tertiary">
        per month, billed annually
      </Text>
    </div>
  ),
};

// Status Messages
export const StatusMessages: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <div>
        <Text variant="body2" weight="semibold" style={{ color: '#00ff00' }}>
          ✓ Success
        </Text>
        <Text variant="caption" color="secondary">
          Your changes have been saved successfully
        </Text>
      </div>
      
      <div>
        <Text variant="body2" weight="semibold" style={{ color: '#ffaa00' }}>
          ⚠ Warning
        </Text>
        <Text variant="caption" color="secondary">
          This action cannot be undone
        </Text>
      </div>
      
      <div>
        <Text variant="body2" weight="semibold" style={{ color: '#ff4444' }}>
          ✗ Error
        </Text>
        <Text variant="caption" color="secondary">
          Failed to save changes. Please try again.
        </Text>
      </div>
      
      <div>
        <Text variant="body2" weight="semibold" style={{ color: '#00aaff' }}>
          ℹ Information
        </Text>
        <Text variant="caption" color="secondary">
          This feature is currently in beta
        </Text>
      </div>
    </div>
  ),
};

// Long Content Example
export const LongContentExample: Story = {
  render: () => (
    <div style={{ maxWidth: '800px' }}>
      <Text variant="h1" style={{ marginBottom: '1rem' }}>
        Typography System
      </Text>
      
      <Text variant="body1" style={{ marginBottom: '2rem' }}>
        A comprehensive typography system ensures consistent and accessible text across your application. 
        This example demonstrates how different text variants work together to create a cohesive reading experience.
      </Text>
      
      <Text variant="h3" style={{ marginBottom: '1rem' }}>
        Introduction
      </Text>
      
      <Text variant="body1" style={{ marginBottom: '1rem' }}>
        Typography is the art and technique of arranging type to make written language legible, readable, 
        and appealing when displayed. The arrangement of type involves selecting typefaces, point sizes, 
        line lengths, line-spacing, and letter-spacing, and adjusting the space between pairs of letters.
      </Text>
      
      <Text variant="body2" color="secondary" style={{ marginBottom: '2rem' }}>
        Good typography enhances readability and comprehension, making content more accessible and enjoyable 
        for users across different devices and contexts.
      </Text>
      
      <Text variant="h4" style={{ marginBottom: '0.5rem' }}>
        Best Practices
      </Text>
      
      <Text variant="caption" color="tertiary" style={{ display: 'block', fontStyle: 'italic' }}>
        Updated October 2025
      </Text>
    </div>
  ),
};
