-- (AS SUPERUSER)
-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE e_user_activation_state AS ENUM (
  'pending',
  'active'
);
CREATE TYPE e_user_reset_password_state AS ENUM (
  'never-yet',
  'pending',
  'in-progress',
  'done'
);

-- equivalent to use of SERIAL or BIGSERIAL
CREATE SEQUENCE users_id_seq
  START WITH 1
  INCREMENT BY 1
  NO MAXVALUE
  NO MINVALUE
  CACHE 1
;

CREATE TABLE users (
  id BIGINT NOT NULL PRIMARY KEY DEFAULT nextval('users_id_seq'),
  uuid UUID NOT NULL DEFAULT uuid_generate_v4(),
  name CHARACTER VARYING(64) NULL,
  username CHARACTER VARYING(32) NULL,
  email CHARACTER VARYING(128) UNIQUE NOT NULL,
  password BYTEA NOT NULL,
  access_token CHARACTER VARYING(256) NULL,
  access_token_issued_at TIMESTAMP WITHOUT TIME ZONE NULL,
  activation_state e_user_activation_state NOT NULL DEFAULT 'pending',
  activation_token CHARACTER VARYING(256) NULL,
  activation_token_expires_at TIMESTAMP WITHOUT TIME ZONE NULL,
  activation_token_sent_at TIMESTAMP WITHOUT TIME ZONE NULL,
  reset_password_state e_user_reset_password_state NOT NULL DEFAULT 'never-yet',
  reset_password_token CHARACTER VARYING(256) NULL,
  reset_password_token_expires_at TIMESTAMP WITHOUT TIME ZONE NULL,
  reset_password_token_sent_at TIMESTAMP WITHOUT TIME ZONE NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc'),
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc')
);

ALTER SEQUENCE users_id_seq OWNED BY users.id;

CREATE UNIQUE INDEX users_access_token_idx ON users(access_token);
CREATE UNIQUE INDEX users_activation_token_idx ON users(activation_token);
CREATE UNIQUE INDEX users_email_idx ON users(email);
CREATE UNIQUE INDEX users_reset_password_token_idx ON users(
  reset_password_token);
CREATE UNIQUE INDEX users_uuid_idx ON users(uuid);

CREATE INDEX users_activation_state_idx ON users(activation_state);
CREATE INDEX users_reset_password_state_idx ON users(reset_password_state);
CREATE INDEX users_username_idx ON users(username);
