-- PromptArk preview schema. Not the Spring Flyway `pl` database.
CREATE TABLE IF NOT EXISTS accounts (
  email TEXT PRIMARY KEY,
  password_hash TEXT,
  role TEXT NOT NULL DEFAULT 'user',
  display_name TEXT,
  bio TEXT
);

CREATE TABLE IF NOT EXISTS oauth_accounts (
  provider TEXT NOT NULL,
  provider_uid TEXT NOT NULL,
  email TEXT NOT NULL REFERENCES accounts(email) ON DELETE CASCADE,
  PRIMARY KEY (provider, provider_uid)
);

CREATE TABLE IF NOT EXISTS access_tokens (
  token TEXT PRIMARY KEY,
  email TEXT NOT NULL REFERENCES accounts(email) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS refresh_tokens (
  token TEXT PRIMARY KEY,
  email TEXT NOT NULL REFERENCES accounts(email) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS square_items (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  kind TEXT NOT NULL,
  excerpt TEXT,
  model TEXT,
  member_count BIGINT,
  content TEXT,
  sort_index INT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS publications (
  id TEXT PRIMARY KEY,
  source_id TEXT NOT NULL,
  status TEXT NOT NULL,
  title TEXT,
  content TEXT,
  author_email TEXT
);

CREATE TABLE IF NOT EXISTS favorites (
  email TEXT NOT NULL REFERENCES accounts(email) ON DELETE CASCADE,
  item_id TEXT NOT NULL REFERENCES square_items(id) ON DELETE CASCADE,
  PRIMARY KEY (email, item_id)
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS media_objects (
  id TEXT PRIMARY KEY,
  owner_email TEXT NOT NULL REFERENCES accounts(email) ON DELETE CASCADE,
  object_key TEXT NOT NULL,
  content_type TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS library_changes (
  owner_email TEXT NOT NULL REFERENCES accounts(email) ON DELETE CASCADE,
  id TEXT NOT NULL,
  kind TEXT NOT NULL,
  payload TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  PRIMARY KEY (owner_email, id)
);
