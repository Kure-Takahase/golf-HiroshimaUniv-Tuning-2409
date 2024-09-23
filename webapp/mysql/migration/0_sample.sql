-- このファイルに記述されたSQLコマンドが、マイグレーション時に実行されます。

SET GLOBAL slow_query_log = 'ON';
SET GLOBAL slow_query_log_file = '/var/lib/mysql/slow_query.log';
SET GLOBAL long_query_time = 0;

CREATE INDEX idx_users_username ON users (username);
CREATE INDEX idx_users_id ON users (id);
CREATE INDEX idx_dispatchers_userid ON dispatchers (user_id);
