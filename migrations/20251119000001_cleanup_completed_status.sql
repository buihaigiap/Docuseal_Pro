-- Clean up completed status to signed
UPDATE submitters SET status = 'signed' WHERE status = 'completed';
