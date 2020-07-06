-- equivalent to use of SERIAL or BIGSERIAL
CREATE SEQUENCE streams_id_seq
  START WITH 1
  INCREMENT BY 1
  NO MAXVALUE
  NO MINVALUE
  CACHE 1
;

CREATE TABLE streams (
  id BIGINT NOT NULL PRIMARY KEY DEFAULT nextval('streams_id_seq'),
  uuid UUID NOT NULL DEFAULT uuid_generate_v4(),
  name CHARACTER VARYING(64) NOT NULL,
  description CHARACTER VARYING(128) NULL,
  namespace_id BIGINT REFERENCES namespaces (id) MATCH FULL NOT NULL,
  archived_at TIMESTAMP WITHOUT TIME ZONE NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc'),
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc')
);

ALTER SEQUENCE streams_id_seq OWNED BY streams.id;

CREATE UNIQUE INDEX streams_namespace_id_name_idx ON streams(
  namespace_id, name);
CREATE UNIQUE INDEX streams_uuid_idx ON streams(uuid);
