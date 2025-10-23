# Links Frontend

Modern SolidJS application for URL shortener service.

## Features

- ⚡ **SolidJS** - Fast and reactive UI framework
- 🎨 **TypeScript** - Type-safe development
- 🛣️ **Router** - Client-side routing with lazy loading
- 🎯 **Error Handling** - Global error boundary and utilities
- 📢 **Toast Notifications** - User-friendly notifications
- 🔄 **HTTP Client** - Fetch-based client with retry logic
- 🎨 **CSS Variables** - Theming system
- 📱 **PWA Support** - Progressive Web App capabilities
- 🚀 **Vite** - Lightning-fast build tool

## Project Structure

```
frontend/
├── src/
│   ├── components/        # Reusable UI components
│   │   ├── ui/           # UI components (Toast, etc.)
│   │   └── ErrorBoundary.tsx
│   ├── config/           # Application configuration
│   │   ├── app.config.ts
│   │   └── routes.config.ts
│   ├── pages/            # Page components
│   │   ├── HomePage.tsx
│   │   ├── DashboardPage.tsx
│   │   └── NotFoundPage.tsx
│   ├── services/         # API and business logic
│   │   ├── api.service.ts
│   │   └── pwa.service.ts
│   ├── stores/           # Global state management
│   │   └── toast.store.ts
│   ├── styles/           # Global styles
│   │   ├── app.css
│   │   ├── variables.css
│   │   └── global.css
│   ├── types/            # TypeScript type definitions
│   │   ├── api.types.ts
│   │   └── common.types.ts
│   ├── utils/            # Utility functions
│   │   ├── helpers.ts
│   │   ├── http.client.ts
│   │   └── error.utils.ts
│   ├── App.tsx           # Main application component
│   ├── index.tsx         # Application entry point
│   └── vite-env.d.ts     # Vite type definitions
├── .env.example          # Environment variables template
├── tsconfig.json         # TypeScript configuration
├── vite.config.ts        # Vite configuration
└── package.json          # Dependencies and scripts
```

## Getting Started

### Prerequisites

- Node.js 18+
- npm or pnpm

### Installation

```bash
# Install dependencies
npm install

# Copy environment file
cp .env.example .env

# Edit .env with your configuration
```

### Development

```bash
# Start development server
npm run dev

# Open http://localhost:3000
```

### Build

```bash
# Build for production
npm run build

# Preview production build
npm run serve
```

## Environment Variables

Create a `.env` file based on `.env.example`:

- `VITE_API_BASE_URL` - Backend API URL (default: http://localhost:8000)
- `VITE_API_TIMEOUT` - API request timeout in ms (default: 30000)
- `VITE_APP_VERSION` - Application version
- `VITE_GOOGLE_CLIENT_ID` - Google OAuth client ID (optional)
- `VITE_PWA_DEV` - Enable PWA in development (default: false)

## Core Features

### HTTP Client

Type-safe HTTP client with:
- Automatic retry logic
- Request/response interceptors
- Error handling
- Timeout support

```typescript
import { http } from './utils/http.client';

const response = await http.get<Data>('/api/v1/method');
```

### Toast Notifications

Global toast notification system:

```typescript
import { toastStore } from './stores/toast.store';

toastStore.showSuccess('Operation successful!');
toastStore.showError('Something went wrong');
toastStore.showWarning('Warning message');
toastStore.showInfo('Information');
```

### Error Handling

Global error boundary and utilities:

```typescript
import { handleError, handleApiError } from './utils/error.utils';

try {
  // code
} catch (error) {
  handleError(error, 'Custom error message');
}
```

### Routing

Type-safe routing with lazy loading:

```typescript
import { ROUTES } from './config/routes.config';
import { A } from '@solidjs/router';

<A href={ROUTES.DASHBOARD}>Dashboard</A>
```

## Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run serve` - Preview production build
- `npm run build:analyze` - Build with bundle analyzer

## License

MIT
