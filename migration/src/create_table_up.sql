create table if not exists "user" (
    id uuid not null primary key default gen_random_uuid(),
    name varchar not null
);
create table if not exists "role" (
    id serial not null primary key,
    name varchar not null
);
create table if not exists "user_role_relation" (
    id uuid not null primary key default gen_random_uuid(),
    user_id int not null,
    role_id int not null
);
create table if not exists "post" (
    id uuid not null primary key default gen_random_uuid(),
    name varchar not null,
    author_id int not null
);