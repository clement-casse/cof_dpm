-- Add up migration script here
CREATE TABLE IF NOT EXISTS dice_rolls (
  id SERIAL PRIMARY KEY,
  roll_id uuid NOT NULL,
  dice VARCHAR(5) NOT NULL,
  result BIGINT NOT NULL
);
