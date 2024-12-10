-- welcom 2 penis databas
--                                                                      cock

ALTER TYPE channel_type RENAME TO channel_type_old;
CREATE TYPE channel_type AS ENUM ('TEXT', 'VOICE', 'GROUP', 'DIRECT');

ALTER TABLE channels ALTER COLUMN channel_type SET DEFAULT NULL;
ALTER TABLE channels ALTER COLUMN channel_type TYPE channel_type USING channel_type::text::channel_type;
ALTER TABLE channels ALTER COLUMN channel_type SET DEFAULT 'TEXT';

DROP TYPE channel_type_old;

CREATE TABLE IF NOT EXISTS categories (
    id BIGINT PRIMARY KEY,
    sphere_id BIGINT NOT NULL,
    name VARCHAR(32),
    position INT,
    is_deleted BOOLEAN DEFAULT FALSE,
    FOREIGN KEY (sphere_id) REFERENCES spheres(id) ON DELETE CASCADE ON UPDATE CASCADE
);

ALTER TABLE channels ADD COLUMN IF NOT EXISTS category_id BIGINT DEFAULT NULL REFERENCES categories(id) ON DELETE CASCADE ON UPDATE CASCADE;
