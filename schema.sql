CREATE TABLE actors (
    ap_id varchar not null,
    actor_ref varchar not null,
    is_local bool not null default false,
    public_key_id varchar not null,
    public_key varchar not null,
    private_key varchar,
    inbox_id varchar,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),
    resources varchar[] not null default array[]::varchar[],
    PRIMARY KEY (ap_id)
);

CREATE TABLE follow_activities (
    follower_ap_id varchar not null,
    followee_ap_id varchar not null,
    activity_ap_id varchar not null,
    accepted_at timestamp,
    accept_activity_id varchar,
    created_at timestamp not null default now(),
    updated_at timestamp not null default now(),
    PRIMARY KEY (follower_ap_id, followee_ap_id)
);

create index follow_activities_follower on public.follow_activities (follower_ap_id);
create index follow_activities_followee on public.follow_activities (followee_ap_id);
create index follow_activities_follow_activity on public.follow_activities (activity_ap_id);
