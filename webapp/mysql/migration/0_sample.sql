-- このファイルに記述されたSQLコマンドが、マイグレーション時に実行されます。
CREATE INDEX idx_users_username ON users (username);
CREATE INDEX idx_users_id ON users (id);
CREATE INDEX idx_dispatchers_userid ON dispatchers (user_id);
