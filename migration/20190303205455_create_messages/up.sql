CREATE TYPE e_log_level AS ENUM (
  'debug', 'information', 'warning', 'error', 'critical');
CREATE TYPE e_log_format AS ENUM ('toml');

-- equivalent to use of SERIAL or BIGSERIAL
CREATE SEQUENCE messages_id_seq
  START WITH 1
  INCREMENT BY 1
  NO MAXVALUE
  NO MINVALUE
  CACHE 1
;

CREATE TABLE messages (
  id BIGINT NOT NULL PRIMARY KEY DEFAULT nextval('messages_id_seq'),
  code CHARACTER VARYING(128) NULL,
  lang CHARACTER VARYING(8) NOT NULL DEFAULT 'en',
  level e_log_level NOT NULL DEFAULT 'information',
  format e_log_format NOT NULL DEFAULT 'toml',
  title CHARACTER VARYING(255) NOT NULL,
  content TEXT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc'),
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc')
);

ALTER SEQUENCE messages_id_seq OWNED BY messages.id;

CREATE INDEX messages_level ON messages(level);
