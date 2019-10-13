DROP INDEX IF EXISTS user_emails_user_id_idx;
DROP INDEX IF EXISTS user_emails_type_idx;
DROP INDEX IF EXISTS user_emails_identification_token_idx;
DROP INDEX IF EXISTS user_emails_identification_state_idx;

DROP INDEX IF EXISTS user_emails_email_idx;

DROP TABLE IF EXISTS user_emails;
DROP SEQUENCE IF EXISTS user_emails_id_seq;

DROP TYPE IF EXISTS e_user_email_role;
DROP TYPE IF EXISTS e_user_email_identification_state;
