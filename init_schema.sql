-- Create Users Table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    uuid VARCHAR NOT NULL UNIQUE,
    username VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    country_code VARCHAR NOT NULL,
    phone_number VARCHAR NOT NULL,
    account_status VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_users_uuid ON users(uuid);

-- Create User Verifications Table
CREATE TABLE IF NOT EXISTS user_verifications (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL UNIQUE,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    email_verified_at TIMESTAMP,
    phone_verified BOOLEAN NOT NULL DEFAULT FALSE,
    phone_verified_at TIMESTAMP,
    business_verified BOOLEAN NOT NULL DEFAULT FALSE,
    business_info TEXT,
    verification_code VARCHAR(6),
    CONSTRAINT fk_user_verifications_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);

-- Create User Socials Table
CREATE TABLE IF NOT EXISTS user_socials (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    provider VARCHAR NOT NULL, -- Enum: KAKAO, GOOGLE, APPLE
    provider_id VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_user_socials_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_user_socials_provider_id ON user_socials(provider_id);
