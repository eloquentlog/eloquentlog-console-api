DROP INDEX messages_level;

DROP TABLE IF EXISTS messages;
DROP SEQUENCE IF EXISTS messages_id_seq;

DROP TYPE IF EXISTS e_log_format;
DROP TYPE IF EXISTS e_log_level;
