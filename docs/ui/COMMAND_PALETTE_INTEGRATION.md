# Command Palette Integration Guide

**Adding Cmd+K Global Shortcut to App.tsx**

---

## 📝 Quick Integration

Add the CommandPalette component to your main App.tsx file with global keyboard shortcut.

### Step 1: Import CommandPalette

```tsx
// App.tsx
import { useState, useEffect } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import CommandPalette from './components/CommandPalette';

// Your other imports...
```

### Step 2: Add State

```tsx
function App() {
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  
  // Rest of your component...
}
```

### Step 3: Add Global Keyboard Listener

```tsx
function App() {
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  
  // Global Cmd+K listener
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Check for Cmd+K (Mac) or Ctrl+K (Windows/Linux)
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault(); // Prevent browser's default search
        setIsCommandPaletteOpen(true);
      }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    
    // Cleanup on unmount
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, []);
  
  // Rest of your component...
}
```

### Step 4: Render CommandPalette

```tsx
function App() {
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  
  useEffect(() => {
    // ... keyboard listener code
  }, []);
  
  return (
    <BrowserRouter>
      {/* Your main app content */}
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/conversations/:id" element={<Conversation />} />
        <Route path="/spaces/:id" element={<Space />} />
        {/* Other routes... */}
      </Routes>
      
      {/* Command Palette - renders as portal/modal */}
      <CommandPalette
        isOpen={isCommandPaletteOpen}
        onClose={() => setIsCommandPaletteOpen(false)}
      />
    </BrowserRouter>
  );
}
```

---

## 🎯 Complete Example

```tsx
// App.tsx
import React, { useState, useEffect } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import CommandPalette from './components/CommandPalette';

// Your pages
import Home from './pages/Home';
import Conversation from './pages/Conversation';
import Space from './pages/Space';
import Login from './pages/Login';

// Auth context
import { AuthProvider } from './contexts/AuthContext';
import ProtectedRoute from './components/ProtectedRoute';

function App() {
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  
  // Global Cmd+K (or Ctrl+K) listener
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+K on Mac, Ctrl+K on Windows/Linux
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault(); // Prevent browser default
        setIsCommandPaletteOpen(true);
      }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, []);
  
  return (
    <BrowserRouter>
      <AuthProvider>
        <div className="app">
          <Routes>
            {/* Public routes */}
            <Route path="/login" element={<Login />} />
            
            {/* Protected routes */}
            <Route path="/" element={
              <ProtectedRoute>
                <Home />
              </ProtectedRoute>
            } />
            
            <Route path="/conversations/:id" element={
              <ProtectedRoute>
                <Conversation />
              </ProtectedRoute>
            } />
            
            <Route path="/spaces/:id" element={
              <ProtectedRoute>
                <Space />
              </ProtectedRoute>
            } />
          </Routes>
          
          {/* Command Palette - Global overlay */}
          <CommandPalette
            isOpen={isCommandPaletteOpen}
            onClose={() => setIsCommandPaletteOpen(false)}
          />
        </div>
      </AuthProvider>
    </BrowserRouter>
  );
}

export default App;
```

---

## 🔧 Advanced: Search Button Trigger

Add a search button in your header/navbar that also opens the palette:

```tsx
// components/Header.tsx
import { useState } from 'react';

function Header({ onOpenSearch }: { onOpenSearch: () => void }) {
  return (
    <header className="app-header">
      <div className="header-left">
        <h1>Sutra AI</h1>
      </div>
      
      <div className="header-center">
        <button 
          className="search-trigger"
          onClick={onOpenSearch}
          aria-label="Search (Cmd+K)"
        >
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
            <path d="M9 17A8 8 0 1 0 9 1a8 8 0 0 0 0 16zM19 19l-4.35-4.35" />
          </svg>
          <span>Search...</span>
          <kbd>⌘K</kbd>
        </button>
      </div>
      
      <div className="header-right">
        {/* User menu, etc. */}
      </div>
    </header>
  );
}
```

Then in App.tsx:

```tsx
function App() {
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  
  // ... keyboard listener ...
  
  return (
    <BrowserRouter>
      <Header onOpenSearch={() => setIsCommandPaletteOpen(true)} />
      
      <Routes>...</Routes>
      
      <CommandPalette
        isOpen={isCommandPaletteOpen}
        onClose={() => setIsCommandPaletteOpen(false)}
      />
    </BrowserRouter>
  );
}
```

---

## 🎨 Styling the Search Button

```css
/* Header.css */
.search-trigger {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  background: #f9fafb;
  color: #6b7280;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.search-trigger:hover {
  background: #f3f4f6;
  border-color: #d1d5db;
}

.search-trigger svg {
  color: #9ca3af;
}

.search-trigger kbd {
  padding: 2px 6px;
  background: white;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  font-size: 12px;
  font-family: 'SF Mono', monospace;
}
```

---

## 🧪 Testing the Integration

### Manual Testing

1. **Open with keyboard:**
   - Press `Cmd+K` (Mac) or `Ctrl+K` (Windows)
   - Palette should open
   - Input should be focused

2. **Search functionality:**
   - Type a query
   - Results should appear
   - Navigate with arrow keys
   - Press Enter to select

3. **Close:**
   - Press Esc
   - Click outside modal
   - Palette should close

4. **Button trigger (if added):**
   - Click search button in header
   - Palette should open

### Automated Tests

```tsx
// App.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import App from './App';

describe('Command Palette Integration', () => {
  test('opens with Cmd+K', () => {
    render(<App />);
    
    // Simulate Cmd+K
    fireEvent.keyDown(window, { key: 'k', metaKey: true });
    
    expect(screen.getByPlaceholderText(/search/i)).toBeInTheDocument();
  });
  
  test('closes with Esc', () => {
    render(<App />);
    
    // Open
    fireEvent.keyDown(window, { key: 'k', metaKey: true });
    expect(screen.getByPlaceholderText(/search/i)).toBeInTheDocument();
    
    // Close
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(screen.queryByPlaceholderText(/search/i)).not.toBeInTheDocument();
  });
  
  test('opens with search button', () => {
    render(<App />);
    
    const searchButton = screen.getByLabelText(/search/i);
    fireEvent.click(searchButton);
    
    expect(screen.getByPlaceholderText(/search/i)).toBeInTheDocument();
  });
});
```

---

## ⚡ Performance Considerations

### 1. Lazy Loading (Optional)

If CommandPalette is large, lazy load it:

```tsx
import { lazy, Suspense } from 'react';

const CommandPalette = lazy(() => import('./components/CommandPalette'));

function App() {
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  
  return (
    <BrowserRouter>
      <Routes>...</Routes>
      
      <Suspense fallback={null}>
        {isCommandPaletteOpen && (
          <CommandPalette
            isOpen={isCommandPaletteOpen}
            onClose={() => setIsCommandPaletteOpen(false)}
          />
        )}
      </Suspense>
    </BrowserRouter>
  );
}
```

### 2. Portal Rendering (Optional)

Render palette in a portal for better layering:

```tsx
// CommandPalette.tsx
import { createPortal } from 'react-dom';

export default function CommandPalette({ isOpen, onClose }: Props) {
  if (!isOpen) return null;
  
  return createPortal(
    <div className="command-palette-overlay">
      {/* ... palette content ... */}
    </div>,
    document.body
  );
}
```

---

## 🐛 Troubleshooting

### Issue: Cmd+K opens browser search

**Solution:** Make sure `e.preventDefault()` is called:

```tsx
if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
  e.preventDefault(); // ← This is critical!
  setIsCommandPaletteOpen(true);
}
```

### Issue: Multiple keyboard listeners

**Solution:** Ensure cleanup:

```tsx
useEffect(() => {
  const handler = (e: KeyboardEvent) => { /* ... */ };
  window.addEventListener('keydown', handler);
  return () => window.removeEventListener('keydown', handler); // ← Cleanup
}, []);
```

### Issue: Palette doesn't close

**Solution:** Check that onClose is wired correctly:

```tsx
<CommandPalette
  isOpen={isCommandPaletteOpen}
  onClose={() => setIsCommandPaletteOpen(false)} // ← Must update state
/>
```

### Issue: Can't navigate to results

**Solution:** Ensure API client is configured with base URL:

```tsx
// api/client.ts
import axios from 'axios';

const api = axios.create({
  baseURL: process.env.REACT_APP_API_URL || 'http://localhost:8000',
  headers: {
    'Content-Type': 'application/json'
  }
});

// Add auth token interceptor
api.interceptors.request.use((config) => {
  // httpOnly cookie authentication (v3.0.0 - XSS immune)
  // Tokens stored server-side, automatically sent with every request
  // NO manual token management required!
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

export default api;
```

---

## 📚 Related Documentation

- **Search API:** See `SESSION_3_2_COMPLETE.md` for API details
- **Hook Usage:** See `useSearch.ts` for hook documentation
- **Component Props:** See `CommandPalette.tsx` for prop types
- **Styling:** See `CommandPalette.css` and `SearchResults.css`

---

## ✅ Checklist

Before deploying:

- [ ] CommandPalette imported in App.tsx
- [ ] Global keyboard listener added
- [ ] Cmd+K opens palette
- [ ] Ctrl+K works on Windows/Linux
- [ ] Esc closes palette
- [ ] Click outside closes palette
- [ ] Navigation works (arrow keys, Enter)
- [ ] Search results display correctly
- [ ] Dark mode tested
- [ ] Mobile responsive (if applicable)
- [ ] Tests passing

---

**Ready to search!** 🚀
