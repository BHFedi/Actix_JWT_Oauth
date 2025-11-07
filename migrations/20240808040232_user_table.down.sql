-- Add down migration script here

-- 1. Revert changes to the users table
-- NOTE: We must ensure all entries have a password before setting NOT NULL again, 
-- but for a clean rollback, we usually assume the down script runs against a clean state.
ALTER TABLE users DROP COLUMN IF EXISTS auth_provider;
ALTER TABLE users ALTER COLUMN password SET NOT NULL;

-- 2. Drop OAuth components
DROP TABLE IF EXISTS oauth_accounts;
DROP TYPE IF EXISTS oauth_provider;

-- 3. Drop original users table components
DROP TABLE IF EXISTS "users";
DROP TYPE IF EXISTS user_role;
DROP EXTENSION IF EXISTS "uuid-ossp";