create table alerts (
	id char(32) primary key,
	/* IN_PROGRESS, COMPLETED */
	progress varchar(11) not null,
	cadence varchar(7) not null,
	data text,
	date bigint not null
);

create table alert_preferences (
	id char(32) primary key,
	alerts text,
	model_id char(32) references models (id) not null,
	last_updated bigint not null
);
