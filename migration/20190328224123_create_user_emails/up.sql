CREATE TYPE e_user_email_activation_state AS ENUM ('pending', 'done');
CREATE TYPE e_user_email_role AS ENUM ('general', 'primary');

-- equivalent to use of SERIAL or BIGSERIAL
CREATE SEQUENCE user_emails_id_seq
  START WITH 1
  INCREMENT BY 1
  NO MAXVALUE
  NO MINVALUE
  CACHE 1
;

CREATE TABLE user_emails (
  id BIGINT NOT NULL PRIMARY KEY DEFAULT nextval('user_emails_id_seq'),
  user_id BIGINT REFERENCES users (id) NOT NULL,
  email CHARACTER VARYING(64) NULL,
  role e_user_email_role NOT NULL DEFAULT 'general',
  activation_state e_user_email_activation_state NOT NULL DEFAULT 'pending',
  activation_token CHARACTER VARYING(256) NULL,
  activation_token_expires_at TIMESTAMP WITHOUT TIME ZONE NULL,
  activation_token_granted_at TIMESTAMP WITHOUT TIME ZONE NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc'),
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
    DEFAULT (now() AT TIME ZONE 'utc')
);

ALTER SEQUENCE user_emails_id_seq OWNED BY user_emails.id;

CREATE UNIQUE INDEX user_emails_email_idx ON user_emails(email);

CREATE INDEX user_emails_activation_state_idx ON user_emails(activation_state);
CREATE INDEX user_emails_activation_token_idx ON user_emails(activation_token);
CREATE INDEX user_emails_role_idx ON user_emails(role);
CREATE INDEX user_emails_user_id_idx ON user_emails(user_id);
