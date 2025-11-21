-- Add migration script here

CREATE TABLE IF NOT EXISTS users (
    id bigint PRIMARY KEY not null comment '用户ID',
    ws_id bigint NOT NULL comment '空间ID',
    fullname VARCHAR(30) NOT NULL comment '用户名',
    password VARCHAR(100) NOT NULL comment '密码',
    email VARCHAR(30) NOT NULL comment '邮箱',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP() comment '创建时间',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP() comment '更新时间'
);

create index idx_users_email on users (email);
create index idx_users_ws_id on users (ws_id);

CREATE TABLE IF NOT EXISTS chats (
    id bigint PRIMARY KEY not null comment '聊天ID',
    title VARCHAR(50) NOT NULL comment '标题',
    `type` VARCHAR(20) NOT NULL comment '聊天类型 single group public private',
    members VARCHAR(255) NOT NULL comment '成员ID列表',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP() comment '创建时间',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP() comment '更新时间'
);

CREATE TABLE IF NOT EXISTS chat_members (
    id bigint PRIMARY KEY not null comment '主键ID',
    chat_id bigint NOT NULL comment '聊天ID',
    user_id bigint NOT NULL comment '用户ID'
);
create index idx_chat_members_chat_id_user_id on chat_members (chat_id, user_id);
create index idx_chat_members_user_id_chat_id on chat_members (user_id, chat_id);

CREATE TABLE IF NOT EXISTS messages (
    id bigint PRIMARY KEY not null comment '消息ID',
    chat_id bigint NOT NULL comment '聊天ID',
    sender_id bigint NOT NULL comment '发送者ID',
    content TEXT NOT NULL comment '内容',
    files TEXT NOT NULL comment '文件列表',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP() comment '创建时间'
);

create index idx_messages_chat_id_created_at on messages (chat_id, created_at);

CREATE TABLE IF NOT EXISTS workspaces (
    id bigint PRIMARY KEY not null comment '空间ID',
    name VARCHAR(50) NOT NULL comment '名称',
    description TEXT NOT NULL comment '描述',
    owner_id bigint NOT NULL comment '空间拥有者ID',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP() comment '创建时间',
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP() comment '更新时间'
);

create index idx_workspaces_owner_id on workspaces (owner_id);
