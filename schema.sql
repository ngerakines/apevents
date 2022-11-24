CREATE TABLE users
(
    name varchar not null,
    domain varchar not null,
    public_key varchar not null,
    private_key varchar not null,
    object_id varchar not null,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),
    PRIMARY KEY (name, domain)
);
