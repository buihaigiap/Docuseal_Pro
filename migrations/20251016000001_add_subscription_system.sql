-- Add subscription related fields to users table
ALTER TABLE users
ADD COLUMN subscription_status VARCHAR(20) DEFAULT 'free',
ADD COLUMN subscription_expires_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
ADD COLUMN free_usage_count INTEGER DEFAULT 0;

-- Create payment_records table để track thanh toán
CREATE TABLE payment_records (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    stripe_session_id VARCHAR(100),
    stripe_payment_intent_id VARCHAR(100),
    amount_cents INTEGER NOT NULL,
    currency VARCHAR(3) DEFAULT 'USD',
    status VARCHAR(20) DEFAULT 'pending', -- pending, completed, failed, refunded
    stripe_price_id VARCHAR(100), -- Stripe Price ID từ webhook
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_users_subscription_status ON users(subscription_status);
CREATE INDEX idx_users_subscription_expires_at ON users(subscription_expires_at);
CREATE INDEX idx_users_free_usage_count ON users(free_usage_count);
CREATE INDEX idx_payment_records_user_id ON payment_records(user_id);
CREATE INDEX idx_payment_records_status ON payment_records(status);
CREATE INDEX idx_payment_records_stripe_session_id ON payment_records(stripe_session_id);