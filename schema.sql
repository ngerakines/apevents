CREATE TABLE users.public
(
    name varchar not null,
    domain varchar not null,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),
    PRIMARY KEY (name, domain)
);
