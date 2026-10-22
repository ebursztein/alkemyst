CREATE TABLE chat_conversations (
    ordinal INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE, create_key TEXT NOT NULL UNIQUE CHECK(length(CAST(create_key AS BLOB)) BETWEEN 1 AND 128),
    fingerprint TEXT NOT NULL CHECK(length(fingerprint)=64 AND fingerprint NOT GLOB '*[^0-9a-f]*'),
    created_at INTEGER NOT NULL, deleted INTEGER NOT NULL DEFAULT 0 CHECK(deleted IN (0,1)),
    title TEXT CHECK(title IS NULL OR length(CAST(title AS BLOB)) BETWEEN 1 AND 200),
    version_id TEXT REFERENCES deck_versions(id),
    share_deck INTEGER NOT NULL CHECK(share_deck IN (0,1)),
    share_notes INTEGER NOT NULL CHECK(share_notes IN (0,1)),
    share_history INTEGER NOT NULL CHECK(share_history IN (0,1)),
    CHECK((share_deck=0 AND share_notes=0 AND share_history=0) OR version_id IS NOT NULL),
    CHECK((deleted=0 AND title IS NOT NULL) OR
        (deleted=1 AND title IS NULL AND version_id IS NULL AND share_deck=0 AND share_notes=0 AND share_history=0))
) STRICT;
CREATE INDEX chat_conversation_order ON chat_conversations(deleted,ordinal DESC);
CREATE TABLE chat_messages (
    id TEXT PRIMARY KEY NOT NULL,
    conversation_id TEXT NOT NULL REFERENCES chat_conversations(id),
    sequence INTEGER NOT NULL CHECK(sequence BETWEEN 1 AND 2000),
    idempotency_key TEXT NOT NULL CHECK(length(CAST(idempotency_key AS BLOB)) BETWEEN 1 AND 128),
    role TEXT NOT NULL CHECK(role IN ('user','assistant','tool')),
    body TEXT NOT NULL CHECK(length(CAST(body AS BLOB)) BETWEEN 1 AND 16384),
    created_at INTEGER NOT NULL,
    UNIQUE(conversation_id,sequence), UNIQUE(conversation_id,idempotency_key)
) STRICT;
CREATE TRIGGER chat_inputs_immutable BEFORE UPDATE ON chat_conversations
WHEN old.deleted=1 OR new.deleted!=1 OR new.id IS NOT old.id
    OR new.ordinal IS NOT old.ordinal OR new.create_key IS NOT old.create_key
    OR new.fingerprint IS NOT old.fingerprint OR new.created_at IS NOT old.created_at
BEGIN SELECT RAISE(ABORT,'conversation inputs are immutable'); END;
CREATE TRIGGER chat_tombstone_no_delete BEFORE DELETE ON chat_conversations
BEGIN SELECT RAISE(ABORT,'conversation creation tombstones preserve retry safety'); END;
CREATE TRIGGER chat_message_no_update BEFORE UPDATE ON chat_messages
BEGIN SELECT RAISE(ABORT,'messages are immutable'); END;
CREATE TRIGGER chat_message_contiguous BEFORE INSERT ON chat_messages
WHEN (SELECT deleted FROM chat_conversations WHERE id=new.conversation_id)=1
    OR new.sequence!=coalesce((SELECT max(sequence) FROM chat_messages WHERE conversation_id=new.conversation_id),0)+1
BEGIN SELECT RAISE(ABORT,'messages must append to an active conversation'); END;
CREATE VIEW chat_conversation_details AS SELECT c.*,
    json_object('id',c.id,'title',c.title,'version_id',c.version_id,'created_at',c.created_at,
        'share_deck',json(CASE c.share_deck WHEN 1 THEN 'true' ELSE 'false' END),
        'share_notes',json(CASE c.share_notes WHEN 1 THEN 'true' ELSE 'false' END),
        'share_history',json(CASE c.share_history WHEN 1 THEN 'true' ELSE 'false' END),
        'revision',coalesce((SELECT max(sequence) FROM chat_messages WHERE conversation_id=c.id),0)) AS data
FROM chat_conversations c;
CREATE VIEW chat_message_details AS SELECT m.*,
    json_object('id',m.id,'conversation_id',m.conversation_id,'sequence',m.sequence,
        'role',m.role,'body',m.body,'created_at',m.created_at) AS data
FROM chat_messages m;
PRAGMA user_version=10;
