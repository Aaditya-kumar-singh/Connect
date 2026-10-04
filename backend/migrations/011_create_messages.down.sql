ALTER TABLE conversation_members DROP CONSTRAINT IF EXISTS fk_conversation_members_last_read_message;
DROP TABLE IF EXISTS messages;
