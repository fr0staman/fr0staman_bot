-- Your SQL goes here









CREATE TABLE `grow_log`(
	`id` INTEGER UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
	`game_id` INTEGER NOT NULL,
	`created_at` DATETIME NOT NULL,
	`weight_change` INTEGER NOT NULL,
	`current_weight` INTEGER UNSIGNED NOT NULL,
	FOREIGN KEY (`game_id`) REFERENCES `game`(`id`)
);

