-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
    '4cf11d46-5c74-48c1-8d52-80b05d820bc6',
    'admin',
    '$argon2id$v=19$m=15000,t=2,p=1$5d7MLP3SmN1euQYDrQ76EQ$SanDe+4jHpp0RJMZqyJC8yBM8+ADvve1PZDT3vPQefQ'
);