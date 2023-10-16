-- Your SQL goes here

ALTER TABLE `groups` ADD COLUMN `lang` VARCHAR(2);



ALTER TABLE `inline_users` DROP COLUMN `lang`;



ALTER TABLE `users` ADD COLUMN `lang` VARCHAR(2);

