CREATE TABLE IF NOT EXISTS users (
  id BIGINT PRIMARY KEY,
  username VARCHAR(32) UNIQUE NOT NULL,
  display_name VARCHAR(32),

  -- Thanks Emre, Olivier, Sharp Eyes and Sham.
  social_credit INT NOT NULL DEFAULT 0, -- All hail Xi Jinping
  status VARCHAR(128),
  bio VARCHAR(4096),
  avatar BIGINT,
  banner BIGINT,
  badges BIT VARYING(64) NOT NULL DEFAULT b'0',
  permissions BIT VARYING(64) NOT NULL DEFAULT b'0',
  verified BOOLEAN NOT NULL DEFAULT FALSE,
  email VARCHAR(256) UNIQUE NOT NULL,
  password CHAR(97) NOT NULL, -- The length of the argon2 encoded strings with our configuration
  two_factor_auth VARCHAR(16),
  FOREIGN KEY (avatar) REFERENCES files(id),
  FOREIGN KEY (banner) REFERENCES files(id)
);
