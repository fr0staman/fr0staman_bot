-- This file should undo anything in `up.sql`
CREATE TABLE `counter`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`count` INTEGER NOT NULL
);




ALTER TABLE `inline_groups` DROP COLUMN `invited_at`;




ALTER TABLE `users` DROP COLUMN `started`;
ALTER TABLE `users` DROP COLUMN `subscribed`;
ALTER TABLE `users` DROP COLUMN `supported`;
ALTER TABLE `users` DROP COLUMN `banned`;
ALTER TABLE `users` DROP COLUMN `created_at`;
ALTER TABLE `users` ADD COLUMN `status` TINYINT NOT NULL;

