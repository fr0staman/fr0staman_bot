-- Your SQL goes here

ALTER TABLE `groups` ADD COLUMN `username` VARCHAR(64);
ALTER TABLE `groups` ADD COLUMN `title` VARCHAR(128) NOT NULL DEFAULT '';







ALTER TABLE `users` ADD COLUMN `username` VARCHAR(64);
ALTER TABLE `users` ADD COLUMN `last_name` VARCHAR(64);
ALTER TABLE `users` ADD COLUMN `first_name` VARCHAR(64) NOT NULL DEFAULT '';

