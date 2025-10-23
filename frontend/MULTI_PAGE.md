# Multi-Page Architecture

## 📁 Structure

```
frontend/
├── index.html              # Landing page (root /)
├── admin.html              # Admin panel (/admin)
├── src/
│   ├── landing/           # Landing page (Vanilla TS)
│   │   ├── landing.ts     # Landing logic
│   │   └── landing.css    # Landing styles
│   ├── admin.tsx          # Admin entry point
│   ├── AdminApp.tsx       # Admin SolidJS app
│   ├── pages/             # Admin pages
│   ├── components/        # Shared components
│   ├── services/          # API services
│   ├── utils/             # Utilities
│   └── ...
```

## 🌐 URLs

- **`/`** - Landing page (Vanilla TS, lightweight)
- **`/admin`** - Admin panel (SolidJS, full features)

## 🎯 How It Works

### Landing Page (`/`)
- Pure TypeScript, no frameworks
- Minimal bundle size (~20KB)
- Fast initial load
- URL shortening form
- Features showcase

### Admin Panel (`/admin`)
- Full SolidJS application
- Dashboard, analytics, management
- All admin features
- Code splitting - only loads when accessed

## 🚀 Development

```bash
npm run dev
```

Access:
- Landing: http://localhost:3000/
- Admin: http://localhost:3000/admin.html

## 📦 Production Build

```bash
npm run build
```

Output:
```
dist/
├── index.html          # Landing
├── admin.html          # Admin
├── assets/
│   ├── landing-*.js    # Landing bundle (~20KB)
│   ├── admin-*.js      # Admin bundle
│   └── ...
```

## 🔗 Navigation

From landing to admin:
```html
<a href="/admin">Admin Panel</a>
```

From admin to landing:
```html
<a href="/">Back to Home</a>
```

## ⚡ Performance

- **Landing**: No framework overhead, instant load
- **Admin**: Lazy-loaded SolidJS, code-split by route
- **Shared**: Common utilities, types, API client

## 🎨 Styling

- **Landing**: Standalone CSS file (`landing/landing.css`)
- **Admin**: CSS modules + global styles (`styles/`)
