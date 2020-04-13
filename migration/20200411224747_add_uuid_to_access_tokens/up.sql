ALTER TABLE access_tokens ADD COLUMN uuid UUID NOT NULL
  DEFAULT uuid_generate_v4();

CREATE INDEX access_tokens_uuid_idx ON access_tokens(uuid);
