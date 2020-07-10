CREATE TYPE e_membership_role AS ENUM (
  'primary_owner',
  'owner',
  'member'
);

-- equivalent to use of SERIAL or BIGSERIAL
CREATE SEQUENCE memberships_id_seq
  START WITH 1
  INCREMENT BY 1
  NO MAXVALUE
  NO MINVALUE
  CACHE 1
;

CREATE TABLE memberships (
  id BIGINT NOT NULL PRIMARY KEY DEFAULT nextval('memberships_id_seq'),
  namespace_id BIGINT REFERENCES namespaces (id) MATCH FULL NOT NULL,
  user_id BIGINT REFERENCES users (id) MATCH FULL NOT NULL,
  role e_membership_role NOT NULL DEFAULT 'member',
  revoked_at TIMESTAMP WITHOUT TIME ZONE NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc'),
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc')
);

ALTER SEQUENCE memberships_id_seq OWNED BY memberships.id;
