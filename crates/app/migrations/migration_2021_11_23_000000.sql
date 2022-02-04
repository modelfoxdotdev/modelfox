create table monitors (
	id char(32) primary key,
	model_id char(32) references models (id) not null,
	data text not null,
	cadence integer not null,
	last_checked bigint
);

create table alerts (
	id char(32) primary key,
	monitor_id char(32) references monitors (id) not null,
	data text not null,
	date bigint not null
);

create table alert_sends (
	id char(32) primary key,
	alert_id char(32) references alerts (id) not null,
	attempt_count integer not null,
	method text not null,
	status integer not null,
	initiated_date bigint not null,
	completed_date bigint
);
