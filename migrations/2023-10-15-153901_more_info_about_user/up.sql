-- Your SQL goes here
DROP TABLE IF EXISTS `counter`;

ALTER TABLE `inline_groups` ADD COLUMN `invited_at` DATETIME NOT NULL;

ALTER TABLE `users` ADD COLUMN `started` BOOL NOT NULL DEFAULT 0;
ALTER TABLE `users` ADD COLUMN `subscribed` BOOL NOT NULL DEFAULT 0;
ALTER TABLE `users` ADD COLUMN `supported` BOOL NOT NULL DEFAULT 0;
ALTER TABLE `users` ADD COLUMN `banned` BOOL NOT NULL DEFAULT 0;
ALTER TABLE `users` ADD COLUMN `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP;

UPDATE `users` u JOIN `inline_users` iu ON u.id = iu.uid SET `subscribed` = 1 WHERE iu.`status` = 1;
UPDATE `users` u JOIN `inline_users` iu ON u.id = iu.uid SET `supported` = 1 WHERE iu.`status` = 2;
UPDATE `users` u SET `banned` = 1 WHERE `status` = 1;
-- If user banned me in private, that means, he started also
UPDATE `users` u SET `started` = 1 WHERE `status` = 1;

ALTER TABLE `users` DROP COLUMN `status`;
