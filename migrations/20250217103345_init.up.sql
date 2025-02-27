-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password TEXT NOT NULL,
    dob DATE NOT NULL,
    mobno VARCHAR(15) NOT NULL,
    wallet_address TEXT UNIQUE,
    private_key TEXT 
);

CREATE TABLE wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    currency VARCHAR(10) NOT NULL,
    wallet_type VARCHAR(20) CHECK (wallet_type IN ('hot', 'cold', 'deposit', 'withdrawal')),
    network VARCHAR(20) NOT NULL,
    wallet_address VARCHAR(255) UNIQUE NOT NULL,
    wallet_private_key VARCHAR(255) UNIQUE NOT NULL,
    status VARCHAR(20) CHECK (status IN ('active', 'inactive')) DEFAULT 'active'   
);
