ALTER TABLE messages alter author_id SET DEFAULT 1; -- deleted user id
ALTER TABLE messages DROP CONSTRAINT messages_author_id_fkey;
ALTER TABLE messages ADD CONSTRAINT messages_author_id_fkey FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE SET DEFAULT ON UPDATE CASCADE;

INSERT INTO USERS(id, username, verified, email, password)
VALUES (0, 'eludris', TRUE, 'bob@eludris.com', '');

INSERT INTO USERS(id, username, verified, email, password)
VALUES (1, 'deleted-user', TRUE, 'phantom@eludris.com', '');
