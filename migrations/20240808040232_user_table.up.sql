-- Add up migration script here

-- 1. Original Users Table Setup
CREATE TYPE user_role AS ENUM ('admin', 'moderator', 'user');
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE "users" (
    id UUID NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    photo VARCHAR NOT NULL DEFAULT 'default.png',
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    password VARCHAR(100) NOT NULL,
    role user_role NOT NULL DEFAULT 'user',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX users_email_idx ON users (email);

--- 2. OAuth/Mock Server Setup ---

-- Create oauth_provider enum with only 'mock'
CREATE TYPE oauth_provider AS ENUM ('mock');

-- Create oauth_accounts table
CREATE TABLE IF NOT EXISTS oauth_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider oauth_provider NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Ensure one OAuth account per provider per user
    UNIQUE(user_id, provider),
    -- Ensure one provider account can't be linked to multiple users
    UNIQUE(provider, provider_user_id)
);

-- Add indexes for faster lookups
CREATE INDEX idx_oauth_accounts_user_id ON oauth_accounts(user_id);
CREATE INDEX idx_oauth_accounts_provider_user_id ON oauth_accounts(provider, provider_user_id);

--- 3. Update Existing Users Table for OAuth Support ---

-- Update users table to allow nullable password (for OAuth-only users)
ALTER TABLE users ALTER COLUMN password DROP NOT NULL;

-- Add a column to track if user signed up via OAuth (default is 'local')
ALTER TABLE users ADD COLUMN auth_provider VARCHAR(50) NOT NULL DEFAULT 'local';