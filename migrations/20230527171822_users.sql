CREATE TABLE IF NOT EXISTS users (
    id BIGINT UNSIGNED PRIMARY KEY,
    username VARCHAR(32) UNIQUE NOT NULL,
    display_name VARCHAR(32),

    -- Thanks Emre, Olivier, Sharp Eyes and Sham.
    social_credit INT NOT NULL DEFAULT 0, -- All hail Xi Jinping
    status VARCHAR(128),
    bio VARCHAR(4096),
    avatar BIGINT UNSIGNED,
    banner BIGINT UNSIGNED,
    badges BIT(64) NOT NULL DEFAULT 0,
    permissions BIT(64) NOT NULL DEFAULT 0,
    email VARCHAR(256) UNIQUE NOT NULL,
    password BINARY(32) NOT NULL,
    two_factor_auth VARCHAR(16)
);
