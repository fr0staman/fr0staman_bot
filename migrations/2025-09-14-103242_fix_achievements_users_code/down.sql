-- This file should undo anything in `up.sql`

ALTER TABLE `achievements_users` MODIFY COLUMN `code` TINYINT NOT NULL UNSIGNED;











