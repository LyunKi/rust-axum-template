create table if not exists "user"
(
    id   serial  not null primary key,
    name varchar not null
);

create table if not exists "role"
(
    id   serial  not null primary key,
    name varchar not null
);

create table if not exists "user_role_relation"
(
    id      serial not null primary key,
    user_id int references "user" (id),
    role_id int references "role" (id)
);

create table if not exists "post"
(
    id        serial  not null primary key,
    name      varchar not null,
    author_id int references "user" (id)
);


