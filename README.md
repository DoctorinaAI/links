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
# Server Configuration
CONFIG_ENVIRONMENT=production
CONFIG_LOGS=info
CONFIG_ADDRESS=0.0.0.0:8000

# Database (choose one)
CONFIG_DATABASE_CONNECTION=sqlite://data/links.db?mode=rwc
# CONFIG_DATABASE_CONNECTION=postgresql://user:password@localhost/links

# Google OAuth
CONFIG_GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com

# Email Access Control (comma-separated, supports wildcards)
# Leave empty or omit to allow all authenticated Google users
# Examples:
#   CONFIG_ALLOWED_EMAILS=admin@example.com                    # Single email
#   CONFIG_ALLOWED_EMAILS=*@doctorina.com                      # All emails from domain
#   CONFIG_ALLOWED_EMAILS=admin@example.com,*@company.com      # Multiple patterns
CONFIG_ALLOWED_EMAILS=*@doctorina.com,admin@example.com
```

#### Option B: Command-Line Arguments

```bash
./target/release/server \
  --env production \
  --logs info \
  --address 0.0.0.0:8000 \
  --database-connection "sqlite://data/links.db?mode=rwc" \
  --google-client-id "your-client-id.apps.googleusercontent.com" \
  --allowed-emails "admin@example.com,*@company.com"
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
│   │   └── state.rs           # Shared state
│   ├── config/                # Configuration
│   ├── database/              # Database abstraction
│   ├── models/                # Data models
│   └── services/              # Business logic
│       ├── google_auth_service.rs  # Google OAuth
│       ├── fingerprint_service.rs  # User tracking
│       └── short_link_service.rs   # Link management
├── migrations/                # SQL migrations
├── public/                    # Frontend assets
│   └── index.html            # Web interface
├── logs/                      # Log files (generated)
├── data/                      # SQLite database (generated)
└── Cargo.toml                # Rust dependencies
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