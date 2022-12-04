CREATE TABLE actors (
    ap_id varchar not null,
    actor_ref varchar not null,
    is_local bool not null default false,
    public_key varchar not null,
    private_key varchar,
    inbox_id varchar,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),
    resources varchar[] not null default array[]::varchar[],
    PRIMARY KEY (ap_id)
);

CREATE TABLE actor_followers (
    actor_ap_id varchar not null,
    follower_ap_id varchar not null,
    accept_ap_id varchar,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),
    PRIMARY KEY (actor_ap_id, follower_ap_id)
);
