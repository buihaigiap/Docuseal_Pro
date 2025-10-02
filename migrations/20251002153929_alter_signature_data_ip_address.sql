-- Alter ip_address column from INET to TEXT in signature_data table
ALTER TABLE signature_data ALTER COLUMN ip_address TYPE TEXT;
