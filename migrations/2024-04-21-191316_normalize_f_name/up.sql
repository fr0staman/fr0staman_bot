-- Your SQL goes here

UPDATE `users` u INNER JOIN inline_users iu ON iu.uid = u.id SET u.first_name = iu.f_name;
UPDATE `users` u INNER JOIN game g ON g.uid = u.id SET u.first_name = g.f_name;


ALTER TABLE `game` DROP COLUMN `f_name`;
ALTER TABLE `inline_users` DROP COLUMN `f_name`;




