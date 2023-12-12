-- Add migration script here

create table users(
    id integer primary key autoincrement not null,
    username text not null,
    password text not null
)