
ALTER TABLE message_attachments ADD COLUMN description TEXT, ADD COLUMN spoiler BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE message_attachments RENAME COLUMN attachment_id TO file_id;

ALTER TABLE message_attachments DROP CONSTRAINT message_attachments_attachment_id_fkey;
ALTER TABLE message_attachments ADD CONSTRAINT message_attachments_file_id_fkey FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE  ON UPDATE CASCADE;

ALTER TABLE files DROP COLUMN spoiler;