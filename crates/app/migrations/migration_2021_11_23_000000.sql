create table monitors (
	id char(32) primary key,
	model_id char(32) references models (id) not null,
	data text not null,
	cadence integer not null,
	date bigint not null
);

create table alerts (
	id char(32) primary key,
	monitor_id char(32) references monitors (id) not null,
	data text not null,
	date bigint not null
);
