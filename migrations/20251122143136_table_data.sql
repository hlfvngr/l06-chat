-- Add migration script here
insert into workspaces (id, name, description, owner_id) values (1, 'default', '默认空间', 0);

insert into users (id, ws_id, fullname, password, email) values (1, 1, 'admin', 'admin', 'admin@admin');
insert into users (id, ws_id, fullname, password, email) values (2, 1, 'user', 'user', 'user@user');
insert into users (id, ws_id, fullname, password, email) values (3, 1, 'test', 'test', 'test@test');
insert into users (id, ws_id, fullname, password, email) values (4, 1, 'demo', 'demo', 'demo@demo');

insert into chats (id, title, `type`) values (1, '默认群聊', 'group');
insert into chats (id, title, `type`) values (2, '凡人修仙传', 'group');

insert into chat_members (chat_id, user_id) values (1, 1);
insert into chat_members (chat_id, user_id) values (1, 2);
insert into chat_members (chat_id, user_id) values (1, 3);

insert into chat_members (chat_id, user_id) values (2, 2);
insert into chat_members (chat_id, user_id) values (2, 4);
