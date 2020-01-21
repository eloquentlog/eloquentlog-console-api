CREATE TYPE e_access_token_state AS ENUM (
  'disabled',
  'enabled'
);

-- {CLIENT,PERSONAL}_ACCESS_TOKEN
CREATE TYPE e_agent_type AS ENUM (
  'client',
  'person'
);

-- equivalent to use of SERIAL or BIGSERIAL
CREATE SEQUENCE access_tokens_id_seq
  START WITH 1
  INCREMENT BY 1
  NO MAXVALUE
  NO MINVALUE
  CACHE 1
;

CREATE TABLE access_tokens (
  id BIGINT NOT NULL PRIMARY KEY DEFAULT nextval('access_tokens_id_seq'),
  agent_id BIGINT NOT NULL,
  agent_type e_agent_type NOT NULL DEFAULT 'client',
  name CHARACTER VARYING(64) NOT NULL,
  token BYTEA NULL,
  state e_access_token_state NOT NULL DEFAULT 'disabled',
  revoked_at TIMESTAMP WITHOUT TIME ZONE NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc'),
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc')
);

ALTER SEQUENCE access_tokens_id_seq OWNED BY access_tokens.id;

CREATE UNIQUE INDEX access_tokens_agent_idx ON access_tokens(agent_id, agent_type);
CREATE UNIQUE INDEX access_tokens_token_idx ON access_tokens(token);

CREATE INDEX access_tokens_state_idx ON access_tokens(state);
