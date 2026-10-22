UPDATE mutation_receipts SET body=json_remove(body, '$.origin') WHERE json_type(body, '$.origin') IS NOT NULL;
PRAGMA user_version=5;
