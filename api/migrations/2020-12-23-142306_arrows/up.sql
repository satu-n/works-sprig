CREATE TABLE arrows (
  source INT REFERENCES tasks ON DELETE CASCADE,
  target INT REFERENCES tasks ON DELETE CASCADE,
  PRIMARY KEY (source, target)
);
