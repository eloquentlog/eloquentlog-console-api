DROP INDEX IF EXISTS messages_user_id_idx;
ALTER TABLE messages DROP COLUMN user_id;

-- messages_stream_id_fkey
ALTER TABLE messages ADD COLUMN stream_id BIGINT REFERENCES streams (id)
  MATCH FULL NOT NULL;

CREATE INDEX messages_stream_id_idx ON messages(stream_id);
