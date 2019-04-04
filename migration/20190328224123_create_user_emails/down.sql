DROP INDEX user_emails_user_id;
DROP INDEX user_emails_type;
DROP INDEX user_emails_activation_token;
DROP INDEX user_emails_activation_state;

DROP INDEX user_emails_email;

DROP TABLE IF EXISTS user_emails;
DROP SEQUENCE IF EXISTS user_emails_id_seq;

DROP TYPE IF EXISTS e_user_email_type;
DROP TYPE IF EXISTS e_user_email_activation_state;
