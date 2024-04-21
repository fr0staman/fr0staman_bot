-- This file should undo anything in `up.sql`

ALTER TABLE `game` ADD COLUMN `f_name` VARCHAR(64) NOT NULL;
ALTER TABLE `inline_users` ADD COLUMN `f_name` VARCHAR(64) NOT NULL;


UPDATE `game` g INNER JOIN users u ON u.id = g.uid SET g.f_name = u.first_name;
UPDATE `inline_users` iu INNER JOIN users u ON u.id = iu.uid SET iu.f_name = u.first_name;




