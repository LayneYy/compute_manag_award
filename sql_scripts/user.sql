-- auto-generated definition
create table user
(
    user_id          bigint         not null
        primary key,
    real_name        varchar(15)    null,
    inviter_id       bigint         null,
    superior_id      bigint         null,
    constraint FK4tgp1anovymlbh041vu8r2n3
        foreign key (inviter_id) references user (user_id),
    constraint FK7i9i4civn1mgkpmc6sj6h32xp
        foreign key (superior_id) references user (user_id)
);

create index IDX1fjbtrw33tteq941gx2p3yf7w
    on user (real_name);
