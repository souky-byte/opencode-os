-- Add multi-phase implementation tracking columns to sessions table
ALTER TABLE sessions ADD COLUMN implementation_phase_number INTEGER DEFAULT NULL;
ALTER TABLE sessions ADD COLUMN implementation_phase_title TEXT DEFAULT NULL;
