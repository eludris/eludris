-- welcom 2 penis databas
--                                                                      cock

CREATE OR REPLACE FUNCTION edit_position(IN from_pos NUMERIC, IN to_pos NUMERIC, IN cur_pos NUMERIC)
RETURNS NUMERIC
AS $$
  BEGIN
    IF cur_pos = from_pos THEN
      RETURN to_pos;
    ELSIF from_pos > to_pos THEN
      RETURN cur_pos + (cur_pos BETWEEN to_pos AND from_pos)::int;
    ELSE
      RETURN cur_pos - (cur_pos BETWEEN from_pos AND to_pos)::int;
    END IF;
  END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION edit_channel_position(
  IN from_pos NUMERIC,
  IN to_pos NUMERIC,
  IN cur_pos NUMERIC,
  IN from_cat NUMERIC,
  IN to_cat NUMERIC,
  IN cur_cat NUMERIC
)
RETURNS NUMERIC
AS $$
  BEGIN
    IF cur_cat = from_cat AND cur_pos = from_pos THEN
      RETURN to_pos;
    ELSIF cur_cat = from_cat AND cur_pos > from_pos THEN
      RETURN cur_pos - 1;
    ELSIF cur_cat = to_cat AND cur_pos >= to_pos THEN
      RETURN cur_pos + 1;
    ELSE
      RETURN cur_pos;
    END IF;
  END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION edit_channel_category(
  IN from_pos NUMERIC,
  IN to_pos NUMERIC,
  IN cur_pos NUMERIC,
  IN from_cat NUMERIC,
  IN to_cat NUMERIC,
  IN cur_cat NUMERIC
)
RETURNS NUMERIC
AS $$
  BEGIN
    IF cur_cat = from_cat AND cur_pos = from_pos THEN
      RETURN to_cat;
    ELSE
      RETURN cur_cat;
    END IF;
  END;
$$ LANGUAGE plpgsql;

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
