import os, sqlite3
from pathlib import Path
if os.name != 'nt' or os.environ.get('GITHUB_ACTIONS') != 'true':
    raise SystemExit('Requires a disposable Windows runner')
path = Path(os.environ['APPDATA']) / 'app.promptark.desktop' / 'quick-clips.sqlite'
path.parent.mkdir(parents=True, exist_ok=True)
assert not path.exists(), 'Expected a fresh runner'
with sqlite3.connect(path) as db:
    db.execute("CREATE TABLE clips(id TEXT PRIMARY KEY, kind TEXT NOT NULL, title TEXT NOT NULL, body TEXT, CHECK ((kind='password')=(body IS NULL)))")
    db.execute("INSERT INTO clips VALUES ('legacy', 'text', 'Legacy clip', 'Legacy body')")
