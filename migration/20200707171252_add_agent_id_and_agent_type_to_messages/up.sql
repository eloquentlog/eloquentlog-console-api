ALTER TABLE messages ADD COLUMN agent_id BIGINT NOT NULL;
ALTER TABLE messages ADD COLUMN agent_type e_agent_type NOT NULL DEFAULT 'client';
