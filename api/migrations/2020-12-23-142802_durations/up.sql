CREATE TABLE durations (
  id SERIAL PRIMARY KEY,
  open TIME NOT NULL,
  close TIME NOT NULL,
  owner INT NOT NULL REFERENCES users ON DELETE CASCADE,
  CHECK (open < close),
  UNIQUE (open, close, owner)
);
