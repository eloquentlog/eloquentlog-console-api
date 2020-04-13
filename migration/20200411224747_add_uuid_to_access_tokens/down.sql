DROP INDEX IF EXISTS access_tokens_uuid_idx;

ALTER TABLE access_tokens DROP COLUMN IF EXISTS uuid RESTRICT;
