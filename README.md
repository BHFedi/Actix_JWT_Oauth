# 🦀 Rust Actix-Web Authentication with OAuth2 Mock Server

A complete authentication system built with Rust and Actix-Web featuring traditional email/password authentication and OAuth2 integration with a local mock server for development and testing.

## 📋 Table of Contents

- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Project Structure](#project-structure)
- [Quick Start](#quick-start)
- [OAuth2 Mock Server Setup](#oauth2-mock-server-setup)
- [Configuration](#configuration)
- [Database Setup](#database-setup)
- [API Documentation](#api-documentation)
- [Testing the Application](#testing-the-application)

---

## 🏗️ Architecture

```
┌─────────────────┐
│   Frontend      │
│  (React/etc)    │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────────────────────┐
│           Actix-Web Server (Port 8080)              │
├─────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │   Auth API   │  │  OAuth API   │  │  User API │ │
│  │              │  │              │  │           │ │
│  │ /register    │  │ /authorize   │  │ /me       │ │
│  │ /login       │  │ /callback    │  │ /users    │ │
│  │ /logout      │  │              │  │           │ │
│  └──────────────┘  └──────────────┘  └───────────┘ │
│                                                      │
│  ┌────────────────────────────────────────────────┐ │
│  │           JWT Middleware & Guards              │ │
│  └────────────────────────────────────────────────┘ │
│                                                      │
│  ┌────────────────────────────────────────────────┐ │
│  │              Database Layer (SQLx)             │ │
│  └────────────────────────────────────────────────┘ │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │   PostgreSQL DB      │
            │                      │
            │  - users table       │
            │  - oauth_accounts    │
            └──────────────────────┘

         ┌──────────────────────────┐
         │  OAuth2 Mock Server      │
         │  (Port 8081)             │
         │                          │
         │  - /authorize            │
         │  - /token                │
         │  - /userinfo             │
         └──────────────────────────┘
```

---

## 📦 Prerequisites

- **Rust** 1.70+ ([Install Rust](https://rustup.rs/))
- **PostgreSQL** 14+ ([Install PostgreSQL](https://www.postgresql.org/download/))
- **SQLx CLI** for migrations
  ```bash
  cargo install sqlx-cli --features postgres
  ```
- **OAuth2 Mock Server** (instructions below)
- **Node.js** (optional, for frontend)

---

## 📁 Project Structure

```
rust_auth/
├── src/
│   ├── handlers/           # API route handlers
│   │   ├── mod.rs
│   │   ├── auth.rs         # Email/password auth endpoints
│   │   ├── oauth.rs        # OAuth2 endpoints
│   │   └── users.rs        # User management endpoints
│   ├── utils/              # Utility functions
│   │   ├── mod.rs
│   │   ├── password.rs     # Password hashing/verification
│   │   ├── token.rs        # JWT signing/verification
│   │   └── oauth_client.rs # OAuth2 client utilities
│   ├── auth.rs             # JWT middleware
│   ├── config.rs           # Environment configuration
│   ├── db.rs               # Database access layer
│   ├── dtos.rs             # Data Transfer Objects
│   ├── error.rs            # Error handling
│   ├── main.rs             # Application entry point
│   └── models.rs           # Database models
├── migrations/             # Database migrations
│   ├── xxxxx_initial_setup.up.sql
│   └── xxxxx_initial_setup.down.sql
├── .env                    # Environment variables
├── Cargo.toml              # Dependencies
└── README.md               # This file
```

---

## 🚀 Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/rust_auth.git
cd rust_auth
```

### 2. Install Dependencies

```bash
cargo build
```

### 3. Set Up Environment Variables

Create a `.env` file in the project root:

```properties
# Database Configuration
DATABASE_URL=postgresql://postgres:password@localhost:5432/rust_auth?schema=public

# JWT Configuration
JWT_SECRET_KEY=my_ultra_secure_jwt_secret_key_change_this_in_production
JWT_MAXAGE=60

# Application URL
APP_URL=http://localhost:8080

# Mock OAuth2 Server Configuration
MOCK_OAUTH_CLIENT_ID=actix_dev_client
MOCK_OAUTH_CLIENT_SECRET=actix_dev_secret_key
MOCK_OAUTH_SERVER_BASE_URL=http://localhost:8081
MOCK_OAUTH_AUTHORIZE_URL=http://localhost:8081/dev-issuer/authorize
MOCK_OAUTH_TOKEN_URL=http://localhost:8081/dev-issuer/token
MOCK_OAUTH_USERINFO_URL=http://localhost:8081/dev-issuer/userinfo
MOCK_OAUTH_REDIRECT_URI=http://localhost:8080/api/oauth/mock/callback

# Frontend Redirect (after successful OAuth login)
OAUTH_SUCCESS_REDIRECT=http://localhost:3000/dashboard
```

### 4. Set Up PostgreSQL Database

```bash
# Create database
createdb rust_auth

# Or using psql
psql -U postgres
CREATE DATABASE rust_auth;
\q
```

### 5. Run Database Migrations

```bash
sqlx migrate run
```

### 6. Start the Application

```bash
cargo run
```

The server will start on `http://localhost:8080`

---

## 🎭 OAuth2 Mock Server Setup

For development and testing, you need a local OAuth2 server.

### Configuring Your Mock Server

Your mock server must provide these endpoints:

1. **Authorization Endpoint**: `GET /dev-issuer/authorize`
   - Accepts: `client_id`, `redirect_uri`, `scope`, `state`, `response_type`
   - Returns: Authorization code via redirect

2. **Token Endpoint**: `POST /dev-issuer/token`
   - Accepts: `code`, `client_id`, `client_secret`, `redirect_uri`, `grant_type`
   - Returns: JSON with `access_token`, `token_type`, `expires_in`, etc.

3. **UserInfo Endpoint**: `GET /dev-issuer/userinfo`
   - Accepts: `Authorization: Bearer <token>` header
   - Returns: JSON with user info (`sub`, `name`, `email`, `picture`, etc.)

### Registering Your Client

In your mock OAuth2 server, register a client with these settings:

- **Client ID**: `actix_dev_client`
- **Client Secret**: `actix_dev_secret_key`
- **Redirect URI**: `http://localhost:8080/api/oauth/mock/callback`
- **Scopes**: `openid`, `email`, `profile`
- **Grant Types**: `authorization_code`

---

## ⚙️ Configuration

### Environment Variables Explained

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@localhost:5432/db` |
| `JWT_SECRET_KEY` | Secret key for JWT signing | `your-secret-key` |
| `JWT_MAXAGE` | JWT expiration time (minutes) | `60` |
| `APP_URL` | Your application's base URL | `http://localhost:8080` |
| `MOCK_OAUTH_CLIENT_ID` | OAuth client ID | `actix_dev_client` |
| `MOCK_OAUTH_CLIENT_SECRET` | OAuth client secret | `actix_dev_secret_key` |
| `MOCK_OAUTH_AUTHORIZE_URL` | OAuth authorization endpoint | `http://localhost:8081/dev-issuer/authorize` |
| `MOCK_OAUTH_TOKEN_URL` | OAuth token endpoint | `http://localhost:8081/dev-issuer/token` |
| `MOCK_OAUTH_USERINFO_URL` | OAuth userinfo endpoint | `http://localhost:8081/dev-issuer/userinfo` |
| `MOCK_OAUTH_REDIRECT_URI` | OAuth callback URL | `http://localhost:8080/api/oauth/mock/callback` |
| `OAUTH_SUCCESS_REDIRECT` | Frontend redirect after login | `http://localhost:3000/dashboard` |

---

## 🗄️ Database Setup

### Schema Overview

#### Users Table

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(100),  -- Nullable for OAuth-only users
    role user_role NOT NULL DEFAULT 'user',
    photo VARCHAR NOT NULL DEFAULT 'default.png',
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    auth_provider VARCHAR(50) NOT NULL DEFAULT 'local',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

#### OAuth Accounts Table

```sql
CREATE TABLE oauth_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider oauth_provider NOT NULL,  -- ENUM: 'mock'
    provider_user_id VARCHAR(255) NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, provider),
    UNIQUE(provider, provider_user_id)
);
```

### Running Migrations

```bash
# Run all pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

---

## 📚 API Documentation

### Interactive Documentation

Once the server is running, access Swagger UI at:

```
http://localhost:8080/swagger-ui/
```

### Authentication Endpoints

#### 1. Register User

**POST** `/api/auth/register`

```json
{
  "name": "John Doe",
  "email": "john@example.com",
  "password": "password123",
  "passwordConfirm": "password123"
}
```

**Response (201 Created):**
```json
{
  "status": "success",
  "data": {
    "user": {
      "id": "uuid",
      "name": "John Doe",
      "email": "john@example.com",
      "role": "user",
      "photo": "default.png",
      "verified": false,
      "createdAt": "2024-01-01T00:00:00Z",
      "updatedAt": "2024-01-01T00:00:00Z"
    }
  }
}
```

#### 2. Login User

**POST** `/api/auth/login`

```json
{
  "email": "john@example.com",
  "password": "password123"
}
```

**Response (200 OK):**
```json
{
  "status": "success",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

Sets `token` cookie with JWT.

#### 3. Logout User

**POST** `/api/auth/logout`

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Response (200 OK):**
```json
{
  "status": "success"
}
```

Clears the `token` cookie.

### OAuth2 Endpoints

#### 1. Initiate OAuth Login

**GET** `/api/oauth/mock/authorize`

Redirects to the OAuth2 authorization server.

**Example:**
```bash
curl http://localhost:8080/api/oauth/mock/authorize
# Redirects to: http://localhost:8081/dev-issuer/authorize?client_id=...&redirect_uri=...
```

#### 2. OAuth Callback (Handled Automatically)

**GET** `/api/oauth/mock/callback?code=xxx&state=xxx`

This endpoint is called by the OAuth server after user authorization. It:
1. Exchanges the authorization code for an access token
2. Fetches user info from the OAuth server
3. Creates/links user account in the database
4. Generates a JWT token
5. Redirects to the frontend with the JWT token

**Frontend Redirect:**
```
http://localhost:3000/dashboard?token=eyJhbGc...
```

---

## 🧪 Testing the Application

### 1. Manual Testing with cURL

#### Test Registration
```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test User",
    "email": "test@example.com",
    "password": "password123",
    "passwordConfirm": "password123"
  }'
```

#### Test Login
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "password123"
  }'
```

#### Test OAuth Flow
```bash
# Step 1: Initiate OAuth (will redirect)
curl -L http://localhost:8080/api/oauth/mock/authorize

# Follow the redirects manually or use a browser
```

### 2. Testing with Swagger UI

1. Navigate to `http://localhost:8080/swagger-ui/`
2. Try the `/api/auth/register` endpoint
3. Try the `/api/auth/login` endpoint (copy the returned token)
4. Click "Authorize" button (top right)
5. Enter: `Bearer <your_token>`
6. Try protected endpoints like `/api/auth/logout`

### 3. Testing OAuth Flow

1. Open browser: `http://localhost:8080/api/oauth/mock/authorize`
2. You'll be redirected to mock OAuth server
3. Approve the authorization (depends on your mock server UI)
4. You'll be redirected back with a JWT token
5. Check browser console or URL for the token

### 4. Database Verification

```sql
-- Check created users
SELECT id, name, email, auth_provider, verified FROM users;

-- Check OAuth accounts
SELECT user_id, provider, provider_user_id FROM oauth_accounts;
```

---
