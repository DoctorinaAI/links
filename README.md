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
    Client[Web Browser] -->|HTTPS| Server[Axum Web Server]
    Server -->|JWT Validation| Google[Google OAuth]
    Server -->|Store/Retrieve| DB[(SQLite/PostgreSQL)]

    subgraph "API v1"
        Public[Public Routes<br/>/health, /link/:slug]
        Admin[Admin Routes<br/>/admin/links]
    end

    Server --> Public
    Server --> Admin

    Admin -->|Requires| Auth[Google Auth Middleware]
```

## 🚀 Quick Start

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
| `GET` | `/link/:slug` | Resolve short link | ❌ |
| `POST` | `/click/:slug` | Track click event | ❌ |

### Admin Routes (`/api/v1/admin`)

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `GET` | `/check` | Auth check | ✅ Google |
| `GET` | `/links` | List all user's links | ✅ Google |
| `POST` | `/links` | Create new short link | ✅ Google |
| `GET` | `/links/:slug` | Get link details | ✅ Google |
| `PUT` | `/links/:slug` | Update link | ✅ Google |
| `DELETE` | `/links/:slug` | Delete link | ✅ Google |
| `GET` | `/links/:slug/stats` | Get click statistics | ✅ Google |

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
    Google->>Browser: Return JWT (ID Token)
    Browser->>Server: API Request with JWT
    Server->>Google: Validate JWT (fetch public keys)
    Google->>Server: Public keys (cached 1h)
    Server->>Server: Verify signature & claims
    Server->>Browser: Authenticated response
```

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
2. **JWT Validation** - All tokens validated server-side with Google's public keys
3. **Email Verification** - Only verified Google emails allowed
4. **Key Caching** - Google public keys cached for 1 hour
5. **Rate Limiting** - Consider adding rate limiting middleware for production
6. **CORS** - Configure CORS appropriately for your domain

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
├── public/                    # Frontend assets (generated)
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