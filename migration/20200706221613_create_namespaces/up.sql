-- equivalent to use of SERIAL or BIGSERIAL
CREATE SEQUENCE namespaces_id_seq
  START WITH 1
  INCREMENT BY 1
  NO MAXVALUE
  NO MINVALUE
  CACHE 1
;

CREATE TABLE namespaces (
  id BIGINT NOT NULL PRIMARY KEY DEFAULT nextval('namespaces_id_seq'),
  uuid UUID NOT NULL DEFAULT uuid_generate_v4(),
  name CHARACTER VARYING(64) NOT NULL,
  description CHARACTER VARYING(256) NULL,
  streams_count INTEGER NOT NULL DEFAULT 0,
  archived_at TIMESTAMP WITHOUT TIME ZONE NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc'),
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc')
);

ALTER SEQUENCE namespaces_id_seq OWNED BY namespaces.id;

CREATE UNIQUE INDEX namespaces_uuid_idx ON namespaces(uuid);
CREATE UNIQUE INDEX namespaces_name_idx ON namespaces(name);
