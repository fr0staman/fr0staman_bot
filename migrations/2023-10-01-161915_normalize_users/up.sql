-- Your SQL goes here

-- Temporary skip for updating after adding
SET FOREIGN_KEY_CHECKS = 0;

ALTER TABLE `users` MODIFY COLUMN `id` INTEGER UNSIGNED NOT NULL AUTO_INCREMENT;

INSERT IGNORE INTO `users` (`user_id`, `status`) (SELECT DISTINCT (user_id), 0 FROM game);
INSERT IGNORE INTO `users` (`user_id`, `status`) (SELECT DISTINCT (user_id), 0 FROM inline_users);
INSERT IGNORE INTO `users` (`user_id`, `status`) (SELECT DISTINCT (user_id), 0 FROM inline_voices);

ALTER TABLE `game` ADD COLUMN `uid` INTEGER UNSIGNED NOT NULL;
ALTER TABLE `game` ADD FOREIGN KEY (uid) REFERENCES `users`(id);

ALTER TABLE `inline_users` ADD COLUMN `uid` INTEGER UNSIGNED NOT NULL;
ALTER TABLE `inline_users` ADD FOREIGN KEY (uid) REFERENCES `users`(id);

ALTER TABLE `inline_voices` ADD COLUMN `uid` INTEGER UNSIGNED NOT NULL;
ALTER TABLE `inline_voices` ADD FOREIGN KEY (uid) REFERENCES `users`(id);

UPDATE `game` g JOIN `users` u ON g.`user_id` = u.`user_id` SET g.`uid` = u.`id`;
UPDATE `inline_users` iu JOIN `users` u ON iu.`user_id` = u.`user_id` SET iu.`uid` = u.`id`;
UPDATE `inline_voices` iv JOIN `users` u ON iv.`user_id` = u.`user_id` SET iv.`uid` = u.`id`;

ALTER TABLE `game` DROP COLUMN `user_id`;
ALTER TABLE `inline_users` DROP COLUMN `user_id`;
ALTER TABLE `inline_voices` DROP COLUMN `user_id`;

SET FOREIGN_KEY_CHECKS = 1;