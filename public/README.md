# Links - Interactive Web Interface

Beautiful and interactive web interface for managing short links with Google authentication.

## Features

✅ **Google OAuth Authentication** - Secure sign-in with Google
✅ **Create Short Links** - Instantly shorten URLs
✅ **View All Links** - See all your shortened links
✅ **Click Statistics** - Track link usage
✅ **Copy to Clipboard** - Easy one-click copying
✅ **Delete Links** - Remove unwanted links
✅ **Responsive Design** - Works on all devices
✅ **Persistent Sessions** - Token saved in localStorage
✅ **Error Handling** - Automatic re-authentication on 401/403

## Setup Instructions

### 1. Configure Google Client ID

Open `public/index.html` and replace `YOUR_CLIENT_ID.apps.googleusercontent.com` with your actual Google Client ID:

```html
<div id="g_id_onload"
     data-client_id="123456789-abcdefg.apps.googleusercontent.com"
     data-callback="handleCredentialResponse"
     data-auto_prompt="false">
</div>
```

### 2. Configure API Endpoint

If your API is not running on `http://localhost:8000`, update the `API_BASE` constant in the script:

```javascript
const API_BASE = 'https://your-api-domain.com/api/v1';
```

### 3. Update Google Cloud Console

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Navigate to "APIs & Services" → "Credentials"
3. Select your OAuth 2.0 Client ID
4. Add your domain to "Authorized JavaScript origins":
   - For local development: `http://localhost:8000`
   - For production: `https://yourdomain.com`

### 4. Serve the HTML File

#### Option 1: Using the Rust server (Recommended)

If your Axum server serves static files:

```rust
// In your server configuration
use tower_http::services::ServeDir;

let app = Router::new()
    .nest_service("/", ServeDir::new("public"))
    .nest("/api", api_routes);
```

#### Option 2: Simple HTTP Server

```bash
# Python 3
python3 -m http.server 8080 --directory public

# Node.js (npx)
npx http-server public -p 8080

# PHP
php -S localhost:8080 -t public
```

Then open `http://localhost:8080` in your browser.

## API Endpoints Required

The interface expects these endpoints to be available:

### GET /api/v1/private/links
Returns list of user's links
```json
{
  "success": true,
  "data": {
    "links": [
      {
        "id": "abc123",
        "slug": "abc123",
        "url": "https://example.com",
        "target_url": "https://example.com",
        "clicks": 42,
        "created_at": "2025-10-22T10:00:00Z"
      }
    ]
  }
}
```

### POST /api/v1/private/links
Creates a new short link
```json
// Request
{
  "url": "https://example.com",
  "slug": null  // auto-generate
}

// Response
{
  "success": true,
  "data": {
    "id": "xyz789",
    "slug": "xyz789",
    "url": "https://example.com",
    "short_url": "http://localhost:8000/xyz789"
  }
}
```

### DELETE /api/v1/private/links/:slug
Deletes a link
```json
{
  "success": true,
  "message": "Link deleted successfully"
}
```

## User Flow

1. **Not Authenticated**
   - User sees Google Sign-In button
   - Clicks button and authenticates with Google
   - JWT token received and stored in localStorage

2. **Authenticated**
   - User info displayed (name, email, picture)
   - Statistics shown (total links, total clicks)
   - Can create new short links
   - Can view all their links
   - Can copy links to clipboard
   - Can delete links

3. **Session Management**
   - Token persists across page refreshes
   - Automatic re-authentication on 401/403 errors
   - Manual sign-out clears all data

## Error Handling

### 401 Unauthorized
- Token expired or invalid
- User automatically signed out
- Auth screen shown with error message

### 403 Forbidden
- Email not in allowed list
- User automatically signed out
- Auth screen shown with error message

### Other Errors
- Displayed inline
- User can retry the action
- Session maintained

## Customization

### Colors

Edit the CSS variables in the `<style>` section:

```css
/* Primary gradient */
background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);

/* Button color */
.btn {
    background: #667eea;
}
```

### Features

You can easily add or remove features by modifying the JavaScript:

- **Disable auto-refresh**: Remove `fetchLinks()` call in `showUserSection()`
- **Add custom slug**: Change the `createLink()` function to include slug input
- **Add link editing**: Implement PUT endpoint and edit button
- **Add analytics**: Track events with Google Analytics or similar

## Browser Compatibility

- ✅ Chrome 90+
- ✅ Firefox 88+
- ✅ Safari 14+
- ✅ Edge 90+

Requires:
- ES6+ support
- Fetch API
- LocalStorage
- Clipboard API

## Security Features

1. **HTTPS Required** (in production)
   - Google OAuth requires HTTPS
   - Tokens transmitted securely

2. **XSS Protection**
   - All user input escaped with `escapeHtml()`
   - No `innerHTML` with user data

3. **Token Storage**
   - Stored in localStorage (consider httpOnly cookies for production)
   - Cleared on sign-out

4. **Server-Side Validation**
   - All JWT validation happens on server
   - Client-side parsing only for display

## Production Checklist

- [ ] Replace `YOUR_CLIENT_ID` with actual Google Client ID
- [ ] Update `API_BASE` to production URL
- [ ] Enable HTTPS
- [ ] Configure CORS on server
- [ ] Add Google domain to authorized origins
- [ ] Test authentication flow
- [ ] Test error scenarios (401, 403, network errors)
- [ ] Test on multiple browsers and devices
- [ ] Consider adding analytics
- [ ] Consider adding CSP headers

## Troubleshooting

### "Missing or invalid Authorization header"
- Token not saved properly
- Check browser console for errors
- Clear localStorage and re-authenticate

### "Access denied for this email address"
- Email not in `CONFIG_ALLOWED_EMAILS`
- Add email to server configuration

### Google Sign-In button doesn't appear
- Check Google Client ID is correct
- Check domain is in authorized origins
- Check browser console for errors
- Ensure Google GSI script is loaded

### Links not loading
- Check API endpoint is correct
- Check server is running
- Check CORS configuration
- Check browser console for errors

## Development Tips

1. **Use browser DevTools**
   - Network tab to see API requests
   - Console to see errors
   - Application tab to check localStorage

2. **Test error scenarios**
   - Stop server (network error)
   - Use expired token (401)
   - Use wrong email (403)

3. **Mock API responses**
   - Comment out `apiRequest()` calls
   - Return mock data for faster development

## License

MIT License - see LICENSE file for details
