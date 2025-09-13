-- Your SQL goes here

CREATE TABLE achievements_users (
    `id` INTEGER UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
    `game_id` INTEGER NOT NULL,
    `created_at` DATETIME NOT NULL,
    `code` TINYINT UNSIGNED NOT NULL,
    FOREIGN KEY (`game_id`) REFERENCES game(`id`),
    CONSTRAINT uq_game_code
        UNIQUE (game_id, code)
);
