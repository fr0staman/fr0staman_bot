-- This file should undo anything in `up.sql`

ALTER TABLE `groups` DROP COLUMN `lang`;



ALTER TABLE `inline_users` ADD COLUMN `lang` VARCHAR(2) NOT NULL;



ALTER TABLE `users` DROP COLUMN `lang`;

