-- This file should undo anything in `up.sql`
SET FOREIGN_KEY_CHECKS = 0;

ALTER TABLE `groups` DROP FOREIGN KEY fk_ig_id;
ALTER TABLE `groups` DROP COLUMN `ig_id`;

SET FOREIGN_KEY_CHECKS = 1;
