-- Your SQL goes here
SET FOREIGN_KEY_CHECKS = 0;

ALTER TABLE `groups` ADD COLUMN `ig_id` INTEGER;
ALTER TABLE `groups` ADD CONSTRAINT fk_ig_id FOREIGN KEY (`ig_id`) REFERENCES `inline_groups`(id);

SET FOREIGN_KEY_CHECKS = 1;
