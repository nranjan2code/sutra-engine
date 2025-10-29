/**
 * Storybook Main Configuration
 * Production-grade setup with TypeScript, accessibility addon, and interactions
 */

import type { StorybookConfig } from '@storybook/react-vite';

const config: StorybookConfig = {
  stories: ['../src/**/*.mdx', '../src/**/*.stories.@(js|jsx|mjs|ts|tsx)'],
  
  addons: [
    '@storybook/addon-links',
    '@storybook/addon-essentials',
    '@storybook/addon-interactions',
    '@storybook/addon-a11y', // Accessibility testing
  ],
  
  framework: {
    name: '@storybook/react-vite',
    options: {},
  },
  
  docs: {
    autodocs: 'tag',
  },
  
  typescript: {
    check: false, // Disable type-checking for faster builds
    reactDocgen: 'react-docgen-typescript',
    reactDocgenTypescriptOptions: {
      shouldExtractLiteralValuesFromEnum: true,
      propFilter: (prop) => {
        // Exclude props from node_modules except our own packages
        if (prop.parent) {
          return !prop.parent.fileName.includes('node_modules') || 
                 prop.parent.fileName.includes('@sutra');
        }
        return true;
      },
    },
  },
  
  core: {
    disableTelemetry: true,
  },
};

export default config;
