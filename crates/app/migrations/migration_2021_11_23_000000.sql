create table alerts (
	id char(32) primary key,
	/* IN_PROGRESS, COMPLETED */
	progress varchar(11) not null,
	cadence varchar(7) not null,
	data text,
	date bigint not null
)
