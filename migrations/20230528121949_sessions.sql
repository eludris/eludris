CREATE TABLE IF NOT EXISTS sessions (
  id BIGINT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  platform VARCHAR(32) NOT NULL,
  client VARCHAR(32) NOT NULL,
  ip INET NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE
);
