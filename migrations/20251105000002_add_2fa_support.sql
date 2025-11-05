-- Add 2FA support to users table
-- Migration: 20251105000002_add_2fa_support.sql

ALTER TABLE users
ADD COLUMN two_factor_secret VARCHAR(255),
ADD COLUMN two_factor_enabled BOOLEAN NOT NULL DEFAULT FALSE;