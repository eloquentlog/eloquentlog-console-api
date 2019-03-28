DROP INDEX users_uuid;
DROP INDEX users_email;
DROP INDEX users_access_token;
DROP INDEX users_reset_password_token;
DROP INDEX users_activation_state;
DROP INDEX users_username;

DROP TABLE IF EXISTS users;
DROP SEQUENCE IF EXISTS users_id_seq;

DROP TYPE IF EXISTS account_activation_state;

-- (AS SUPERUSER)
-- DROP EXTENSION IF EXISTS "uuid-ossp";
