CREATE TYPE STATUS AS ENUM ('ONLINE', 'OFFLINE', 'IDLE', 'BUSY');

CREATE TABLE IF NOT EXISTS users (
  id BIGINT PRIMARY KEY,
  username VARCHAR(32) UNIQUE NOT NULL,
  display_name VARCHAR(32),

  -- Thanks Emre, Olivier, Sharp Eyes and Sham.
  social_credit INT NOT NULL DEFAULT 0, -- All hail Xi Jinping
  status VARCHAR(128),
  status_type STATUS DEFAULT 'ONLINE',
  bio VARCHAR(4096),
  avatar BIGINT,
  banner BIGINT,
  badges BIGINT NOT NULL DEFAULT 0,
  permissions BIGINT NOT NULL DEFAULT 0,
  verified BOOLEAN NOT NULL DEFAULT FALSE,
  email VARCHAR(256) UNIQUE NOT NULL,
  password CHAR(97) NOT NULL, -- The length of the argon2 encoded strings with our configuration
  two_factor_auth VARCHAR(16),
  is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
  FOREIGN KEY (avatar) REFERENCES files(id) ON DELETE SET NULL ON UPDATE CASCADE,
  FOREIGN KEY (banner) REFERENCES files(id) ON DELETE SET NULL ON UPDATE CASCADE
);
