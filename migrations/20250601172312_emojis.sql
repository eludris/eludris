CREATE TABLE IF NOT EXISTS emojis (
  id BIGINT PRIMARY KEY,
  sphere_id BIGINT NOT NULL,
  name VARCHAR(32) NOT NULL,
  file_id BIGINT NOT NULL,
  uploader_id BIGINT NOT NULL DEFAULT 1, -- deleted user id
  is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
  FOREIGN KEY (sphere_id) REFERENCES spheres(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (uploader_id) REFERENCES users(id) ON DELETE SET DEFAULT ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS reactions (
  emoji_id BIGINT,
  unicode_emoji VARCHAR(32),
  message_id BIGINT NOT NULL,
  user_id BIGINT NOT NULL,
  FOREIGN KEY (emoji_id) REFERENCES emojis(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE
);
