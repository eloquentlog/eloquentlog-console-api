DROP INDEX IF EXISTS access_tokens_state_idx;

DROP INDEX IF EXISTS access_tokens_agent_idx;
DROP INDEX IF EXISTS access_tokens_token_idx;

DROP TABLE IF EXISTS access_tokens;
DROP SEQUENCE IF EXISTS access_tokens_id_seq;

DROP TYPE IF EXISTS e_agent_type;
DROP TYPE IF EXISTS e_access_token_state;
