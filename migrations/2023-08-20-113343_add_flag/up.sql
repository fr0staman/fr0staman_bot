-- Your SQL goes here

ALTER TABLE `inline_users` ADD COLUMN `flag` VARCHAR(17) NOT NULL;
UPDATE `inline_users` SET flag = lang;
