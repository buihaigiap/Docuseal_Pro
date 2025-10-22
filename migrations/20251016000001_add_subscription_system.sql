-- Create payment_records table để track thanh toán
CREATE TABLE IF NOT EXISTS payment_records (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    stripe_session_id VARCHAR(100),
    amount_cents INTEGER NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(20) DEFAULT 'pending', -- pending, completed, failed, refunded
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes (only if they don't exist)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_users_subscription_status') THEN
        CREATE INDEX idx_users_subscription_status ON users(subscription_status);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_users_subscription_expires_at') THEN
        CREATE INDEX idx_users_subscription_expires_at ON users(subscription_expires_at);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_users_free_usage_count') THEN
        CREATE INDEX idx_users_free_usage_count ON users(free_usage_count);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_payment_records_user_id') THEN
        CREATE INDEX idx_payment_records_user_id ON payment_records(user_id);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_payment_records_status') THEN
        CREATE INDEX idx_payment_records_status ON payment_records(status);
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_payment_records_stripe_session_id') THEN
        CREATE INDEX idx_payment_records_stripe_session_id ON payment_records(stripe_session_id);
    END IF;
END $$;