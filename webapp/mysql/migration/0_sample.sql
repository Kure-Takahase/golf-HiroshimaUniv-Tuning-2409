-- このファイルに記述されたSQLコマンドが、マイグレーション時に実行されます。
CREATE INDEX idx_last_name ON users (username);
CREATE INDEX idx_last_name ON users (id);
CREATE INDEX idx_last_name ON dispatchers (user_id);
