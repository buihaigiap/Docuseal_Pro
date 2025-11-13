-- Add decline_reason column to submitters table
ALTER TABLE submitters
ADD COLUMN IF NOT EXISTS decline_reason TEXT;

-- Add comment for documentation
COMMENT ON COLUMN submitters.decline_reason IS 'Reason provided by submitter when declining to sign the document';