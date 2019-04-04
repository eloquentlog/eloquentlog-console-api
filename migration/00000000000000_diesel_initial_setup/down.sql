-- diesel
DROP FUNCTION IF EXISTS diesel_manage_updated_at(_tbl regclass);
DROP FUNCTION IF EXISTS diesel_set_updated_at();

-- uuid_generate_v4()
DROP EXTENSION IF EXISTS "uuid-ossp";
