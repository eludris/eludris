-- # DAMATASE CREATED
--
-- This damatabse is really cool
-- it reminds me of going to school
-- and when I go home it always feels
-- like
--
--
-- YIPPEEEEEEEEE

CREATE TYPE sphere_type AS ENUM ('CHAT', 'FORUM', 'HYBRID');

CREATE TABLE IF NOT EXISTS spheres (
  id BIGINT PRIMARY KEY,
  owner_id BIGINT NOT NULL,
  name VARCHAR(32),
  slug VARCHAR(32) UNIQUE NOT NULL,
  sphere_type sphere_type NOT NULL DEFAULT 'HYBRID',
  description VARCHAR(4096),
  icon BIGINT,
  banner BIGINT,
  badges BIGINT NOT NULL DEFAULT 0,
  default_permissions BIGINT NOT NULL DEFAULT 0,
  is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
  FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (icon) REFERENCES files(id) ON DELETE SET NULL ON UPDATE CASCADE,
  FOREIGN KEY (banner) REFERENCES files(id) ON DELETE SET NULL ON UPDATE CASCADE
);

CREATE TYPE channel_type AS ENUM ('CATEGORY', 'TEXT', 'VOICE', 'GROUP', 'DIRECT');

CREATE TABLE IF NOT EXISTS channels (
  id BIGINT PRIMARY KEY,
  sphere BIGINT,
  owner_id BIGINT,
  recipient_id BIGINT,
  channel_type channel_type NOT NULL DEFAULT 'TEXT',
  position SMALLINT,
  icon BIGINT,
  name VARCHAR(32),
  topic VARCHAR(4096),
  default_permissions BIGINT,
  is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
  FOREIGN KEY (sphere) REFERENCES spheres(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (recipient_id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS members (
  id BIGINT NOT NULL,
  sphere BIGINT NOT NULL,
  nickname VARCHAR(32),
  server_avatar BIGINT,
  is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
  FOREIGN KEY (id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (sphere) REFERENCES spheres(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (server_avatar) REFERENCES files(id) ON DELETE SET NULL ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS channel_members (
  id BIGINT NOT NULL,
  channel BIGINT,
  FOREIGN KEY (id) REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (channel) REFERENCES channels(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS messages (
  id BIGINT PRIMARY KEY,
  channel BIGINT NOT NULL,
  content TEXT,
  reference BIGINT,
  is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
  FOREIGN KEY (channel) REFERENCES channels(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (reference) REFERENCES messages(id) ON DELETE SET NULL ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS message_attachments (
  message_id BIGINT,
  attachment_id BIGINT,
  FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (attachment_id) REFERENCES files(id) ON DELETE CASCADE ON UPDATE CASCADE
);
