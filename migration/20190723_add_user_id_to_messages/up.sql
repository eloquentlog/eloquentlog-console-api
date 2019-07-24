-- messages_user_id_fkey
ALTER TABLE messages ADD COLUMN user_id BIGINT REFERENCES users (id)
  MATCH FULL NOT NULL;

CREATE INDEX messages_user_id_idx ON messages(user_id);
