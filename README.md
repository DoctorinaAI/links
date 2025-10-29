# Links 🔗

A high-performance URL shortener service built with Rust and Axum, featuring Google OAuth authentication, analytics, and multi-database support.

## ✨ Features

- 🚀 **Fast & Lightweight** - Built with Rust for maximum performance
- 🔐 **Google OAuth Authentication** - Secure sign-in with Google accounts
- 📊 **Analytics** - Track clicks and user behavior
- 💾 **Multi-Database** - Support for both SQLite and PostgreSQL
- 🌐 **RESTful API** - Clean API with versioning (v1)
- 📚 **Auto-Generated Docs** - Interactive API documentation via Scalar UI
- 🎯 **Email Whitelisting** - Control access with domain wildcards
- 🔄 **Graceful Shutdown** - Proper cleanup on exit
- 📝 **Structured Logging** - JSON logs with daily rotation

## 🏗️ Architecture

```mermaid
graph TB
    Client[Web Browser] -->|1. Google Sign-In| Google[Google OAuth]
    Google -->|2. Google JWT| Client
    Client -->|3. Exchange Token<br/>POST /auth/google| Server[Axum Web Server]
    Server -->|4. Validate| Google
    Server -->|5. Internal JWT<br/>non-expiring| Client
    Client -->|6. API Requests<br/>with JWT| Server
    Server -->|7. Store/Retrieve| DB[(SQLite/PostgreSQL)]

    subgraph "API v1"
        Auth[Auth Routes<br/>/auth/google]
        Public[Public Routes<br/>/health, /link/:slug]
        Admin[Admin Routes<br/>/admin/links]
    end

    Server --> Auth
    Server --> Public
    Server --> Admin

    Admin -->|Requires| AuthMiddleware[JWT Auth Middleware]
```

## 🚀 Quick Start

### TL;DR

```sh
# Release build
cargo build --release

# Then run
./target/release/server --jwt-secret="dev-secret-key-for-local-development" --allowed-emails="*@doctorina.com,plugfox@gmail.com" --google-client-id=123-ABC.apps.googleusercontent.com
```

### Build Profiles

The project includes multiple optimized build profiles for different use cases:

| Profile | Binary Size | Compile Time | Runtime Performance | Use Case |
|---------|-------------|--------------|---------------------|----------|
| `dev` (default) | ~19 MB | ⚡⚡⚡⚡⚡ Very Fast | ⭐⭐⭐ Good | Development & debugging |
| `--release` | ~9 MB | ⚡⚡⚡⚡ Fast | ⭐⭐⭐⭐ Excellent | General production use |
| `--profile release-small` | ~4.5 MB | ⚡⚡ Slow | ⭐⭐⭐ Good | Docker images, size-constrained environments |
| `--profile release-fast` | ~7.5 MB | ⚡ Very Slow | ⭐⭐⭐⭐⭐ Maximum | High-performance production servers |

**Profile configurations:**
- **`dev`**: Basic optimizations (opt-level=1), fast incremental builds, debug symbols included
- **`release`**: Balanced optimization, thin LTO, parallel codegen (16 units), stripped symbols
- **`release-small`**: Maximum size reduction (opt-level="z"), fat LTO, single codegen unit
- **`release-fast`**: Maximum performance (opt-level=3), fat LTO, overflow checks disabled

### Prerequisites

- **Rust** 1.75+ (install from [rustup.rs](https://rustup.rs/))
- **Database**: SQLite (no setup) or PostgreSQL
- **Google Cloud Project** with OAuth 2.0 credentials

### 1. Clone & Build

```bash
git clone https://github.com/DoctorinaAI/links.git
cd links
cargo build --release
```

### 2. Configure Google OAuth

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing one
3. Enable **Google Identity Services** API
4. Navigate to **APIs & Services** → **Credentials**
5. Create **OAuth 2.0 Client ID**:
   - Application type: **Web application**
   - Authorized JavaScript origins: `http://localhost:8000` (dev) or your domain
   - Authorized redirect URIs: Not needed for ID tokens
6. Copy the **Client ID** (format: `xxxxx.apps.googleusercontent.com`)

### 3. Configuration

You can configure the application via **environment variables** or **command-line arguments**.

#### Option A: Environment Variables (`.env` file)

Create a `.env` file in the project root:

```env
# Environment configuration template for the Links application
CONFIG_ENVIRONMENT=development

# Logging level: trace, debug, info, warn, error
CONFIG_LOGS=info

# Server address and port to bind to
CONFIG_ADDRESS=0.0.0.0:8000

# Database connection string
# Could be either SQLite or PostgreSQL
# Examples:
#   - SQLite: sqlite://data/links.db?mode=rwc
#   - PostgreSQL: postgresql://user:password@localhost/dbname
CONFIG_DATABASE_CONNECTION=sqlite://data/links.db?mode=rwc

# Google OAuth Configuration (REQUIRED)
# Get your Client ID from https://console.cloud.google.com/apis/credentials
# This is required for the server to start
CONFIG_GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com

# JWT Secret Key (OPTIONAL but RECOMMENDED for production)
# Used to sign internal non-expiring JWT tokens
# Minimum 16 characters, recommended 32+ for security
# If not set or too short, a random secret will be generated on startup
# WARNING: Changing this will invalidate all existing user sessions
# Generate with: openssl rand -base64 32
CONFIG_JWT_SECRET=your-secret-key-at-least-32-characters-long-change-me-in-production

# Email Access Control (OPTIONAL)
# Comma-separated list of allowed email addresses or domain wildcards
# Leave empty to allow all authenticated Google users
# Examples:
#   Single email:     CONFIG_ALLOWED_EMAILS=admin@example.com
#   Domain wildcard:  CONFIG_ALLOWED_EMAILS=*@example.com
#   Multiple:         CONFIG_ALLOWED_EMAILS=admin@example.com,*@company.com
#   All users:        CONFIG_ALLOWED_EMAILS=
CONFIG_ALLOWED_EMAILS=*@doctorina.com,plugfox@gmail.com

# CORS Configuration (OPTIONAL)
# Comma-separated list of allowed origins for Cross-Origin requests
# Leave empty to allow all origins (permissive mode - recommended for development)
# For production, specify exact origins for better security
# Examples:
#   All origins:      CONFIG_CORS_ORIGINS=
#   Single origin:    CONFIG_CORS_ORIGINS=https://example.com
#   Multiple:         CONFIG_CORS_ORIGINS=https://example.com,https://app.example.com
CONFIG_CORS_ORIGINS=
```

#### Option B: Command-Line Arguments

```bash
./target/release/server \
  --env production \
  --logs info \
  --address 0.0.0.0:8000 \
  --database-connection "sqlite://data/links.db?mode=rwc" \
  --google-client-id "your-client-id.apps.googleusercontent.com" \
  --jwt-secret "your-secret-key-at-least-32-characters-long" \
  --allowed-emails "admin@example.com,*@company.com" \
  --cors-origins "https://example.com,https://app.example.com"
```

### 4. Run

```bash
cargo run --release --bin server
```

The server will start on `http://localhost:8000` (or your configured address).

## 📖 API Documentation

Interactive API documentation is available at:

```
http://localhost:8000/scalar
```

Features:
- 🎨 Beautiful UI with Scalar
- 🔍 Search and filter endpoints
- 🧪 Try API requests directly from browser
- 📋 Request/response examples
- 🔐 Authentication testing

OpenAPI spec available at: `http://localhost:8000/api-docs/openapi.json`

## 🛣️ API Endpoints

### Public Routes (`/api/v1`)

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/health` | Health check | ❌ |
| `GET` | `/about` | Service information | ❌ |
| `POST` | `/auth/google` | Exchange Google JWT for internal token | ❌ |
| `GET` | `/link/:slug` | Resolve link | ❌ |
| `POST` | `/click/:slug` | Track click event | ❌ |
| `PUT` | `/analytics` | Create fingerprint for tracking | ❌ |
| `POST` | `/analytics` | Get parameters for fingerprint | ❌ |

### Admin Routes (`/api/v1/admin`)

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/check` | Auth check | ✅ Internal JWT |
| `GET` | `/links` | List all user's links | ✅ Internal JWT |
| `POST` | `/links` | Create new link | ✅ Internal JWT |
| `GET` | `/links/:slug` | Get link details | ✅ Internal JWT |
| `PUT` | `/links/:slug` | Update link | ✅ Internal JWT |
| `DELETE` | `/links/:slug` | Delete link | ✅ Internal JWT |
| `GET` | `/links/:slug/stats` | Get click statistics | ✅ Internal JWT |

## 🔐 Authentication

### Google OAuth Flow

```mermaid
sequenceDiagram
    participant User
    participant Browser
    participant Server
    participant Google

    User->>Browser: Click "Sign in with Google"
    Browser->>Google: Request ID Token
    Google->>Browser: Return JWT (Google ID Token)
    Browser->>Server: POST /api/v1/auth/google<br/>{token: "google-jwt"}
    Server->>Google: Validate JWT (fetch public keys)
    Google->>Server: Public keys (cached 1h)
    Server->>Server: Verify signature & claims
    Server->>Server: Generate internal JWT<br/>(non-expiring, signed with secret)
    Server->>Browser: {token: "internal-jwt", user: {...}}
    Browser->>Browser: Store internal JWT
    Browser->>Server: API requests with<br/>Authorization: Bearer internal-jwt
    Server->>Server: Verify internal JWT
    Server->>Browser: Authenticated response
```

### Token Exchange

The authentication flow uses a two-step token exchange:

1. **Google Authentication**: User signs in with Google and receives a Google ID Token (JWT)
2. **Token Exchange**: Frontend sends Google JWT to `/api/v1/auth/google`
3. **Validation**: Server validates Google JWT with Google's public keys (cached for 1 hour)
4. **Internal Token**: Server generates a **non-expiring** internal JWT signed with secret key
5. **API Access**: Frontend uses internal JWT for all subsequent API requests

**Example Request:**
```bash
curl -X POST http://localhost:8000/api/v1/auth/google \
  -H "Content-Type: application/json" \
  -d '{"token": "eyJhbGciOiJSUzI1NiIsImtpZCI6..."}'
```

**Example Response:**
```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "123456789",
      "provider": "google",
      "email": "user@example.com",
      "name": "John Doe",
      "picture": "https://lh3.googleusercontent.com/a/..."
    }
  }
}
```

**Internal JWT Claims:**
- `sub` - User ID from Google
- `provider` - Authentication provider (always "google")
- `email` - User's email address
- `email_verified` - Email verification status
- `name` - User's full name
- `picture` - Profile picture URL
- `iat` - Issued at timestamp
- **No `exp`** - Token never expires (invalidated only on server restart or JWT secret change)

### Email Whitelisting

Control who can access admin endpoints using email patterns:

```env
# Specific emails
CONFIG_ALLOWED_EMAILS=admin@example.com,john@example.com

# Domain wildcard (all emails from domain)
CONFIG_ALLOWED_EMAILS=*@company.com

# Mixed patterns
CONFIG_ALLOWED_EMAILS=ceo@example.com,*@company.com,*@partner.com

# Allow all authenticated users (empty or omit variable)
# CONFIG_ALLOWED_EMAILS=
```

**How it works:**
- `*@domain.com` - Allows any email ending with `@domain.com`
- `user@domain.com` - Allows only this specific email
- Multiple patterns separated by commas
- Empty list or omitted variable - Allows all authenticated Google users

### CORS Configuration

Control which origins can access your API using Cross-Origin Resource Sharing (CORS):

```env
# Allow all origins (permissive - for development)
CONFIG_CORS_ORIGINS=

# Allow specific origin
CONFIG_CORS_ORIGINS=https://example.com

# Allow multiple origins
CONFIG_CORS_ORIGINS=https://example.com,https://app.example.com,https://admin.example.com
```

**How it works:**
- **Empty list** (default) - Permissive mode, allows requests from any origin
- **Specific origins** - Only listed origins can access the API
- All HTTP methods are allowed (GET, POST, PUT, DELETE, etc.)
- All headers are allowed
- Credentials (cookies, authorization headers) are supported

**Security recommendations:**
- 🔓 **Development**: Use empty list for convenience
- 🔒 **Production**: Always specify exact origins for security
- ✅ Use HTTPS origins only in production
- ❌ Avoid wildcards in production

## 💾 Database

### SQLite (Default)

Perfect for development and small deployments:

```env
CONFIG_DATABASE_CONNECTION=sqlite://data/links.db?mode=rwc
```

Features:
- ✅ Zero configuration
- ✅ File-based storage
- ✅ Automatic migrations
- ⚠️ Single writer limitation

### PostgreSQL

Recommended for production:

```env
CONFIG_DATABASE_CONNECTION=postgresql://user:password@localhost:5432/links
```

Features:
- ✅ High concurrency
- ✅ Better performance at scale
- ✅ Advanced features
- ✅ Replication support

### Migrations

Database schema is automatically migrated on startup. Migration files are in `migrations/` directory.

## 🔧 Configuration Reference

### Environment Variables

| Variable | CLI Argument | Default | Description |
|----------|--------------|---------|-------------|
| `CONFIG_ENVIRONMENT` | `--env` | `development` | Environment mode |
| `CONFIG_LOGS` | `--logs` | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `CONFIG_ADDRESS` | `--address` | `0.0.0.0:8000` | Server bind address |
| `CONFIG_DATABASE_CONNECTION` | `--database-connection` | `sqlite://data/links.db?mode=rwc` | Database connection string |
| `CONFIG_GOOGLE_CLIENT_ID` | `--google-client-id` | **Required** | Google OAuth Client ID |
| `CONFIG_JWT_SECRET` | `--jwt-secret` | Auto-generated | JWT secret for signing internal tokens (min 16 chars, 32+ recommended) |
| `CONFIG_ALLOWED_EMAILS` | `--allowed-emails` | `[]` (empty = all users) | Comma-separated email patterns |
| `CONFIG_CORS_ORIGINS` | `--cors-origins` | `[]` (empty = all origins) | Comma-separated CORS allowed origins |

### CLI Help

```bash
./target/release/server --help
```

## 📊 Logging

Logs are written to:
- **Console**: Human-readable format with colors
- **Files**: `logs/` directory in JSON format
- **Rotation**: Daily rotation, keeps 7 days

Log levels (from most to least verbose):
1. `trace` - Very detailed debugging
2. `debug` - Debugging information
3. `info` - General information (default)
4. `warn` - Warnings
5. `error` - Errors only

## 🧪 Development

### Run in Development Mode

```bash
cargo run --bin server
```

### Build for Production

```bash
# Standard release build (recommended)
cargo build --release --bin server

# Or with CPU-specific optimizations (faster but not portable)
RUSTFLAGS="-C target-cpu=native" cargo build --release --bin server

# Or for smallest binary
cargo build --profile release-small --bin server

# On Windows PowerShell
$env:RUSTFLAGS="-C target-cpu=native"; cargo build --release --bin server
```

**Binary locations:**
- Release: `./target/release/server.exe`
- Small: `./target/release-small/server.exe`
- Fast: `./target/release-fast/server.exe`

### Run Tests

```bash
cargo test
```

### Lint & Format

```bash
cargo clippy
cargo fmt
```

### Watch Mode (Auto-rebuild)

```bash
cargo install cargo-watch
cargo watch -x run
```

## 🐳 Docker Deployment

Build Docker image:

```bash
docker build -t links:latest .
```

Run with Docker Compose:

```bash
docker-compose up -d
```

## 🔒 Security Considerations

1. **HTTPS Required** - Always use HTTPS in production
2. **Two-Step Authentication**:
   - Google JWT validated with Google's public keys
   - Internal JWT signed with server's secret key
3. **Token Security**:
   - Google tokens: Short-lived, validated server-side
   - Internal tokens: Non-expiring, signed with secret key
   - Tokens invalidated on server restart or JWT secret change
4. **JWT Secret Management**:
   - Use strong secrets (32+ characters recommended)
   - Auto-generated if not provided or too short
   - Store securely, never commit to version control
   - Change periodically in production
5. **Email Verification** - Only verified Google emails allowed
6. **Key Caching** - Google public keys cached for 1 hour
7. **Rate Limiting** - Consider adding rate limiting middleware for production
8. **CORS** - Configure CORS appropriately for your domain

## 📈 Performance

TODO: Add benchmarks and performance metrics here.

## 🛠️ Tech Stack

- **Language**: Rust 2024 Edition
- **Web Framework**: [Axum](https://github.com/tokio-rs/axum) 0.8
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **Database**: [SQLx](https://github.com/launchbadge/sqlx) with SQLite/PostgreSQL
- **Authentication**: [jsonwebtoken](https://github.com/Keats/jsonwebtoken)
- **API Docs**: [utoipa](https://github.com/juhaku/utoipa) + [Scalar](https://scalar.com/)
- **Logging**: [tracing](https://github.com/tokio-rs/tracing)

## 📝 Project Structure

```
links/
├── bin/
│   └── server.rs              # Main server entry point
├── src/
│   ├── api/                   # API layer
│   │   ├── middleware.rs      # Auth & logging middleware
│   │   ├── routes_public.rs   # Public endpoints
│   │   ├── routes_private.rs  # Admin endpoints
│   │   ├── server.rs          # Server configuration
│   │   └── state.rs           # Shared state (injected config & services)
│   ├── config/                 # Configuration
│   ├── database/              # Database abstraction
│   ├── models/                # Data models
│   └── services/              # Business logic
├── migrations/                # SQL migrations
├── frontend/                  # Frontend
├── logs/                      # Log files (generated)
├── data/                      # SQLite database (generated)
└── Cargo.toml                 # Rust dependencies
```

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Run linter: `cargo clippy`
6. Format code: `cargo fmt`
7. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👨‍💻 Author

**Mike Matiunin**
- Email: plugfox@gmail.com
- GitHub: [@DoctorinaAI](https://github.com/DoctorinaAI)

## 🔗 Links

- **Repository**: https://github.com/DoctorinaAI/links
- **Homepage**: https://doctorina.com
- **Documentation**: http://localhost:8000/scalar (when running)
- **Issues**: https://github.com/DoctorinaAI/links/issues

---

Made with ❤️ using Rust 🦀
