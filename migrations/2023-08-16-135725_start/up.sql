-- Your SQL goes here
CREATE TABLE `inline_users`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
	`user_id` BIGINT UNSIGNED NOT NULL,
	`f_name` VARCHAR(64) NOT NULL,
	`weight` INTEGER NOT NULL,
	`date` DATE NOT NULL,
	`lang` VARCHAR(2) NOT NULL,
	`win` SMALLINT UNSIGNED NOT NULL DEFAULT 0,
	`rout` SMALLINT UNSIGNED NOT NULL DEFAULT 0,
	`status` TINYINT NOT NULL DEFAULT 0,
	`name` VARCHAR(64) NOT NULL,
	`gifted` BOOL NOT NULL DEFAULT FALSE
);

CREATE TABLE `groups`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
	`chat_id` BIGINT NOT NULL,
	`date` DATETIME NOT NULL,
	`settings` TINYINT NOT NULL DEFAULT 0,
	`top10_setting` INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE `users`(
	`id` SMALLINT UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
	`user_id` BIGINT UNSIGNED NOT NULL,
	`status` TINYINT NOT NULL DEFAULT 0
);

CREATE TABLE `inline_groups`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
	`chat_instance` BIGINT NOT NULL
);

CREATE TABLE `inline_users_groups`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
	`iu_id` INTEGER NOT NULL,
	`ig_id` INTEGER NOT NULL,
	FOREIGN KEY (`iu_id`) REFERENCES `inline_users`(`id`),
	FOREIGN KEY (`ig_id`) REFERENCES `inline_groups`(`id`)
);

CREATE TABLE `counter`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
	`count` INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE `inline_voices`(
	`id` SMALLINT NOT NULL PRIMARY KEY AUTO_INCREMENT,
	`url` VARCHAR(128) NOT NULL,
	`user_id` BIGINT UNSIGNED NOT NULL,
	`caption` VARCHAR(64) NOT NULL,
	`status` SMALLINT NOT NULL
);

CREATE TABLE `game`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
	`user_id` BIGINT UNSIGNED NOT NULL,
	`group_id` INTEGER NOT NULL,
	`mass` INTEGER NOT NULL DEFAULT 0,
	`date` DATE NOT NULL,
	`name` VARCHAR(64) NOT NULL,
	`f_name` VARCHAR(64) NOT NULL,
	FOREIGN KEY (`group_id`) REFERENCES `groups`(`id`)
);

CREATE TABLE `hryak_day`(
	`id` INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
	`iug_id` INTEGER NOT NULL,
	`date` DATE NOT NULL,
	FOREIGN KEY (`iug_id`) REFERENCES `inline_users_groups`(`id`)
);
