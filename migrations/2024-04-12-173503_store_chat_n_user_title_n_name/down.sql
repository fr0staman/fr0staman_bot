-- This file should undo anything in `up.sql`

ALTER TABLE `groups` DROP COLUMN `username`;
ALTER TABLE `groups` DROP COLUMN `title`;







ALTER TABLE `users` DROP COLUMN `username`;
ALTER TABLE `users` DROP COLUMN `last_name`;
ALTER TABLE `users` DROP COLUMN `first_name`;

