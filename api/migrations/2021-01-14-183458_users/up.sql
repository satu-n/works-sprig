CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  email VARCHAR NOT NULL UNIQUE,
  hash VARCHAR NOT NULL,
  name VARCHAR NOT NULL UNIQUE,
  timescale VARCHAR NOT NULL DEFAULT 'D',
  open TIME NOT NULL,
  close TIME NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  CHECK (open < close)
);
SELECT diesel_manage_updated_at('users');