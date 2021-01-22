CREATE TABLE allocations (
  id SERIAL PRIMARY KEY,
  owner INT NOT NULL REFERENCES users ON DELETE CASCADE,
  open TIME NOT NULL,
  hours INT NOT NULL CHECK (hours >= 0 AND hours <= 24)
);
