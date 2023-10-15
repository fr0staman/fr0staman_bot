-- This file should undo anything in `up.sql`

ALTER TABLE `users` MODIFY COLUMN `id` SMALLINT UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `game` ADD COLUMN `user_id` BIGINT UNSIGNED NOT NULL;
ALTER TABLE `game` DROP COLUMN `uid`;

ALTER TABLE `inline_users` ADD COLUMN `user_id` BIGINT UNSIGNED NOT NULL;
ALTER TABLE `inline_users` DROP COLUMN `uid`;

ALTER TABLE `inline_voices` ADD COLUMN `user_id` BIGINT UNSIGNED NOT NULL;
ALTER TABLE `inline_voices` DROP COLUMN `uid`;
