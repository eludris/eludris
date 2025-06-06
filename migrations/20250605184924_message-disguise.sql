CREATE TABLE IF NOT EXISTS message_disguise (
  message_id BIGINT NOT NULL,
  author VARCHAR(32),
  avatar VARCHAR(1024),
  FOREIGN KEY (message_id) REFERENCES messages(id) ON UPDATE CASCADE ON DELETE CASCADE
);
