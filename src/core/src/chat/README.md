# Local deck chat history

Conversation and message operations use the shared SQLite worker, prepared SQL,
table revisions and bounded result cache. Public reads use one prepared domain
statement each, including filtering, joins and cursor pagination. Conversations
page newest first with `c1:` cursors; messages page chronologically with `m1:`
cursors. Each page contains at most 100 records.

A conversation pins an optional sealed deck version and immutable context-sharing
choices. Deck, note and observed-history sharing default to false and require a
selected version. Creating history or appending user text makes no provider call.
Provider configuration, explicit transmission consent, tool execution and the
React/TanStack AI interface are separate integration work.

Creation keys return the original creation response. Appends require the expected
revision and a conversation-scoped key; retries return the original message and
competing writes at the same revision have one winner. User messages contain at
most 16,384 UTF-8 bytes; conversations contain at most 2,000 messages. Clients
cannot supply assistant/tool roles. Chat-history operations are not MCP tools.

Deleting a conversation transactionally removes its messages and clears its title,
version reference and sharing flags. A tombstone retains only the creation key,
request fingerprint, conversation id, ordinal and creation time so delayed retries
cannot recreate it. The fingerprint hashes normalized creation settings, not message
content. Prompt text never enters the shared mutation-receipt table. Deletion is
logical; existing backups and SQLite recovery files may retain earlier content.
Captured games, deck versions and simulation results are unaffected.

Schema 10 is append-only. Restore accepts exact supported schemas 1–9, validates
them first, and applies trusted migrations only to a private copy. The selected
backup is unchanged. Tests cover restore, retry conflicts, deletion, pinned
versions, authentication and indexed plans with 1,000 conversations.
