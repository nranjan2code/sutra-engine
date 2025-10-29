/**
 * Storybook Preview Configuration
 * Global decorators, parameters, and theme setup
 */

import React from 'react';
import type { Preview } from '@storybook/react';
import { ThemeProvider } from '../src/core';
import { holographicTheme, professionalTheme, commandTheme } from '../src/themes';

// Global styles for Storybook
import './storybook.css';

const preview: Preview = {
  parameters: {
    actions: { argTypesRegex: '^on[A-Z].*' },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/,
      },
      expanded: true,
      sort: 'requiredFirst',
    },
    docs: {
      toc: true,
    },
    backgrounds: {
      default: 'dark',
      values: [
        {
          name: 'dark',
          value: '#0a0a0a',
        },
        {
          name: 'light',
          value: '#ffffff',
        },
        {
          name: 'holographic',
          value: '#000000',
        },
      ],
    },
    viewport: {
      viewports: {
        mobile: {
          name: 'Mobile',
          styles: { width: '375px', height: '667px' },
        },
        tablet: {
          name: 'Tablet',
          styles: { width: '768px', height: '1024px' },
        },
        desktop: {
          name: 'Desktop',
          styles: { width: '1440px', height: '900px' },
        },
        wide: {
          name: 'Wide Desktop',
          styles: { width: '1920px', height: '1080px' },
        },
      },
    },
  },
  
  globalTypes: {
    theme: {
      description: 'Sutra UI Theme',
      defaultValue: 'holographic',
      toolbar: {
        title: 'Theme',
        icon: 'paintbrush',
        items: [
          { value: 'holographic', title: 'Holographic', icon: 'star' },
          { value: 'professional', title: 'Professional', icon: 'document' },
          { value: 'command', title: 'Command', icon: 'terminal' },
        ],
        dynamicTitle: true,
      },
    },
  },
  
  decorators: [
    (Story, context) => {
      const themes = {
        holographic: holographicTheme,
        professional: professionalTheme,
        command: commandTheme,
      };
      
      const theme = themes[context.globals.theme as keyof typeof themes] || holographicTheme;
      
      return (
        <ThemeProvider theme={theme}>
          <div
            style={{
              padding: '2rem',
              minHeight: '100vh',
              backgroundColor: theme.tokens.color.background.primary,
              color: theme.tokens.color.text.primary,
            }}
          >
            <Story />
          </div>
        </ThemeProvider>
      );
    },
  ],
};

export default preview;
