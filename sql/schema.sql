create database if not exists main_db;
use main_db;

create table if not exists auth(
	user_id int auto_increment,
	user_email varchar(255) not null,
	user_pass varchar(255) not null,

	primary key(user_id)
);

create table if not exists tasks(
	task_id int auto_increment,
	user_id int not null,
	task_text varchar(512),

	primary key(task_id, user_id),
	foreign key(user_id)
		references auth(user_id)
);