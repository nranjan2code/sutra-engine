/**
 * Input Component Stories
 * Comprehensive showcase of form inputs with validation
 */

import { useState } from 'react';
import type { Meta, StoryObj } from '@storybook/react';
import { Input } from '../primitives/Input';
import type { InputProps } from '../primitives/Input';

const meta = {
  title: 'Components/Input',
  component: Input as any,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['outlined', 'filled', 'unstyled'],
      description: 'Visual style variant',
    },
    label: {
      control: 'text',
      description: 'Label text',
    },
    helperText: {
      control: 'text',
      description: 'Helper text below input',
    },
    error: {
      control: 'text',
      description: 'Error message',
    },
    required: {
      control: 'boolean',
      description: 'Required field indicator',
    },
    disabled: {
      control: 'boolean',
      description: 'Disabled state',
    },
    loading: {
      control: 'boolean',
      description: 'Loading state with spinner',
    },
  },
  args: {
    variant: 'outlined' as const,
    required: false,
    disabled: false,
    loading: false,
  },
} satisfies Meta<InputProps>;

export default meta;
type Story = StoryObj<InputProps>;

// Basic Examples
export const Default: Story = {
  args: {
    label: 'Email',
    placeholder: 'Enter your email',
  },
};

export const WithHelperText: Story = {
  args: {
    label: 'Username',
    placeholder: 'Choose a username',
    helperText: 'Your username will be public',
  },
};

export const Required: Story = {
  args: {
    label: 'Password',
    type: 'password',
    placeholder: 'Enter password',
    required: true,
    helperText: 'Must be at least 8 characters',
  },
};

export const WithError: Story = {
  args: {
    label: 'Email',
    placeholder: 'Enter your email',
    error: 'Please enter a valid email address',
    defaultValue: 'invalid-email',
  },
};

export const Disabled: Story = {
  args: {
    label: 'Disabled Input',
    placeholder: 'Cannot edit this',
    disabled: true,
    defaultValue: 'Disabled value',
  },
};

export const Loading: Story = {
  args: {
    label: 'Username',
    placeholder: 'Checking availability...',
    loading: true,
    defaultValue: 'john_doe',
  },
};

// Variants
export const OutlinedVariant: Story = {
  args: {
    variant: 'outlined' as const,
    label: 'Outlined Input',
    placeholder: 'Default style',
  },
};

export const FilledVariant: Story = {
  args: {
    variant: 'filled' as const,
    label: 'Filled Input',
    placeholder: 'Filled background',
  },
};

export const UnstyledVariant: Story = {
  args: {
    variant: 'unstyled' as const,
    label: 'Unstyled Input',
    placeholder: 'Minimal styling',
  },
};

// With Icons
export const WithStartIcon: Story = {
  args: {
    label: 'Search',
    placeholder: 'Search...',
    startIcon: <span>🔍</span>,
  },
};

export const WithEndIcon: Story = {
  args: {
    label: 'Website',
    placeholder: 'example.com',
    endIcon: <span>🌐</span>,
  },
};

export const WithBothIcons: Story = {
  args: {
    label: 'Amount',
    placeholder: '0.00',
    startIcon: <span>$</span>,
    endIcon: <span>USD</span>,
  },
};

// Input Types
export const AllInputTypes: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem', maxWidth: '400px' }}>
      <Input type="text" label="Text" placeholder="Enter text" />
      <Input type="email" label="Email" placeholder="user@example.com" />
      <Input type="password" label="Password" placeholder="••••••••" />
      <Input type="number" label="Number" placeholder="123" />
      <Input type="tel" label="Phone" placeholder="(555) 123-4567" />
      <Input type="url" label="URL" placeholder="https://example.com" />
      <Input type="date" label="Date" />
      <Input type="time" label="Time" />
    </div>
  ),
};

// Validation Examples
export const EmailValidation: Story = {
  render: () => {
    const [email, setEmail] = useState('');
    const [error, setError] = useState<string>();
    
    const validateEmail = (value: string) => {
      if (!value) return 'Email is required';
      if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)) {
        return 'Please enter a valid email address';
      }
      return undefined;
    };
    
    return (
      <div style={{ maxWidth: '400px' }}>
        <Input
          type="email"
          label="Email Address"
          placeholder="user@example.com"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          validate={validateEmail}
          onValidation={setError}
          error={error}
          required
          helperText="We'll never share your email"
        />
      </div>
    );
  },
};

export const PasswordStrength: Story = {
  render: () => {
    const [password, setPassword] = useState('');
    const [error, setError] = useState<string>();
    
    const validatePassword = (value: string) => {
      if (!value) return 'Password is required';
      if (value.length < 8) return 'Password must be at least 8 characters';
      if (!/[A-Z]/.test(value)) return 'Password must contain an uppercase letter';
      if (!/[a-z]/.test(value)) return 'Password must contain a lowercase letter';
      if (!/[0-9]/.test(value)) return 'Password must contain a number';
      return undefined;
    };
    
    return (
      <div style={{ maxWidth: '400px' }}>
        <Input
          type="password"
          label="Password"
          placeholder="Enter a strong password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          validate={validatePassword}
          onValidation={setError}
          error={error}
          required
          helperText="Must be 8+ characters with uppercase, lowercase, and number"
        />
      </div>
    );
  },
};

export const UsernameAvailability: Story = {
  render: () => {
    const [username, setUsername] = useState('');
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string>();
    
    const checkAvailability = async (value: string) => {
      if (!value) return 'Username is required';
      if (value.length < 3) return 'Username must be at least 3 characters';
      if (!/^[a-zA-Z0-9_]+$/.test(value)) return 'Only letters, numbers, and underscores allowed';
      
      // Simulate API call
      setLoading(true);
      await new Promise(resolve => setTimeout(resolve, 1000));
      setLoading(false);
      
      // Simulate some usernames being taken
      if (['admin', 'user', 'test'].includes(value.toLowerCase())) {
        return 'This username is already taken';
      }
      
      return undefined;
    };
    
    return (
      <div style={{ maxWidth: '400px' }}>
        <Input
          label="Username"
          placeholder="Choose a username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          validate={checkAvailability}
          onValidation={setError}
          error={error}
          loading={loading}
          required
          helperText={!error && username && !loading ? '✓ Username is available' : 'Letters, numbers, and underscores only'}
        />
      </div>
    );
  },
};

// Form Example
export const LoginForm: Story = {
  render: () => {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [emailError, setEmailError] = useState<string>();
    const [passwordError, setPasswordError] = useState<string>();
    
    const validateEmail = (value: string) => {
      if (!value) return 'Email is required';
      if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)) {
        return 'Invalid email format';
      }
      return undefined;
    };
    
    const validatePassword = (value: string) => {
      if (!value) return 'Password is required';
      if (value.length < 8) return 'Password must be at least 8 characters';
      return undefined;
    };
    
    return (
      <div style={{ maxWidth: '400px', display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
        <h2 style={{ margin: 0 }}>Login</h2>
        
        <Input
          type="email"
          label="Email"
          placeholder="user@example.com"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          validate={validateEmail}
          onValidation={setEmailError}
          error={emailError}
          required
          startIcon={<span>✉️</span>}
        />
        
        <Input
          type="password"
          label="Password"
          placeholder="Enter your password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          validate={validatePassword}
          onValidation={setPasswordError}
          error={passwordError}
          required
          startIcon={<span>🔒</span>}
        />
        
        <button
          style={{
            padding: '0.75rem 1.5rem',
            borderRadius: '4px',
            border: 'none',
            background: 'linear-gradient(135deg, #00ffff 0%, #0080ff 100%)',
            color: '#000',
            fontWeight: 'bold',
            cursor: 'pointer',
          }}
          disabled={!!emailError || !!passwordError || !email || !password}
        >
          Sign In
        </button>
      </div>
    );
  },
};

// All States
export const AllStates: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem', maxWidth: '400px' }}>
      <Input
        label="Default State"
        placeholder="Normal input"
      />
      
      <Input
        label="With Value"
        defaultValue="User input here"
      />
      
      <Input
        label="With Helper Text"
        placeholder="Enter text"
        helperText="This is helper text"
      />
      
      <Input
        label="Required Field"
        placeholder="Required"
        required
      />
      
      <Input
        label="With Error"
        placeholder="Invalid input"
        error="This field has an error"
        defaultValue="wrong value"
      />
      
      <Input
        label="Loading State"
        placeholder="Checking..."
        loading
        defaultValue="validating"
      />
      
      <Input
        label="Disabled State"
        placeholder="Cannot edit"
        disabled
        defaultValue="Disabled input"
      />
    </div>
  ),
};
