-- Your SQL goes here

CREATE TABLE `inline_gifs`(
	`id` SMALLINT NOT NULL PRIMARY KEY AUTO_INCREMENT,
	`file_id` VARCHAR(128) NOT NULL,
	`status` SMALLINT NOT NULL DEFAULT 1,
	`uid` INTEGER UNSIGNED NOT NULL,
	FOREIGN KEY (`uid`) REFERENCES `users`(`id`)
);
