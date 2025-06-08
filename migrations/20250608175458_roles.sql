ALTER TABLE channels DROP COLUMN default_permissions;

CREATE TABLE IF NOT EXISTS roles (
  id BIGINT PRIMARY KEY,
  sphere_id BIGINT,
  position INT,
  name VARCHAR(32),
  allowed BIGINT NOT NULL DEFAULT 0,
  denied BIGINT NOT NULL DEFAULT 0,
  is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
  FOREIGN KEY (sphere_id) REFERENCES spheres(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS channel_permission_overrides (
  channel_id BIGINT NOT NULL,
  role_id BIGINT NOT NULL,
  allowed BIGINT NOT NULL DEFAULT 0,
  denied BIGINT NOT NULL DEFAULT 0,
  is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
  FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE ON UPDATE CASCADE,
  FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE ON UPDATE CASCADE
);

INSERT INTO roles (id, sphere_id, name, position)
SELECT id, id, 'everyone', 0
FROM spheres;
