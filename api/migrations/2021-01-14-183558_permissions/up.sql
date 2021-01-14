CREATE TABLE permissions (
  subject INT REFERENCES users ON DELETE CASCADE,
  object INT REFERENCES users ON DELETE CASCADE,
  edit BOOL NOT NULL,
  PRIMARY KEY (subject, object)
);
