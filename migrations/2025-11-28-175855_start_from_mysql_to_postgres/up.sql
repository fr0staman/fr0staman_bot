-- Your SQL goes here
CREATE TABLE "users"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"user_id" BIGINT NOT NULL,
	"started" BOOL NOT NULL,
	"subscribed" BOOL NOT NULL,
	"supported" BOOL NOT NULL,
	"banned" BOOL NOT NULL,
	"created_at" TIMESTAMP NOT NULL,
	"lang" VARCHAR(2),
	"username" VARCHAR(64),
	"last_name" VARCHAR(64),
	"first_name" VARCHAR(64) NOT NULL
);


CREATE TABLE "inline_groups"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"chat_instance" BIGINT NOT NULL,
	"invited_at" TIMESTAMP NOT NULL
);

CREATE TABLE "inline_users"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"weight" INTEGER NOT NULL,
	"date" DATE NOT NULL,
	"win" INTEGER NOT NULL,
	"rout" INTEGER NOT NULL,
	"name" VARCHAR(64) NOT NULL,
	"gifted" BOOL NOT NULL,
	"flag" VARCHAR(17) NOT NULL,
	"uid" INTEGER NOT NULL,
	FOREIGN KEY ("uid") REFERENCES "users"("id")
);

CREATE TABLE "groups"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"chat_id" BIGINT NOT NULL,
	"date" TIMESTAMP NOT NULL,
	"settings" SMALLINT NOT NULL,
	"top10_setting" INTEGER NOT NULL,
	"lang" VARCHAR(2),
	"active" BOOL NOT NULL,
	"ig_id" INTEGER,
	"username" VARCHAR(64),
	"title" VARCHAR(128) NOT NULL,
	FOREIGN KEY ("ig_id") REFERENCES "inline_groups"("id")
);

CREATE TABLE "game"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"group_id" INTEGER NOT NULL,
	"mass" INTEGER NOT NULL,
	"date" DATE NOT NULL,
	"name" VARCHAR(64) NOT NULL,
	"uid" INTEGER NOT NULL,
	FOREIGN KEY ("group_id") REFERENCES "groups"("id"),
	FOREIGN KEY ("uid") REFERENCES "users"("id")
);

CREATE TABLE "grow_log"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"game_id" INTEGER NOT NULL,
	"created_at" TIMESTAMP NOT NULL,
	"weight_change" INTEGER NOT NULL,
	"current_weight" INTEGER NOT NULL,
	FOREIGN KEY ("game_id") REFERENCES "game"("id")
);

CREATE TABLE "inline_gifs"(
	"id" SMALLSERIAL NOT NULL PRIMARY KEY,
	"file_id" VARCHAR(128) NOT NULL,
	"status" SMALLINT NOT NULL,
	"uid" INTEGER NOT NULL,
	"file_unique_id" VARCHAR(64) NOT NULL,
	FOREIGN KEY ("uid") REFERENCES "users"("id")
);

CREATE TABLE "inline_users_groups"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"iu_id" INTEGER NOT NULL,
	"ig_id" INTEGER NOT NULL,
	FOREIGN KEY ("iu_id") REFERENCES "inline_users"("id"),
	FOREIGN KEY ("ig_id") REFERENCES "inline_groups"("id")
);

CREATE TABLE "hryak_day"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"iug_id" INTEGER NOT NULL,
	"date" DATE NOT NULL,
	FOREIGN KEY ("iug_id") REFERENCES "inline_users_groups"("id")
);

CREATE TABLE "inline_voices"(
	"id" SMALLSERIAL NOT NULL PRIMARY KEY,
	"url" VARCHAR(128) NOT NULL,
	"caption" VARCHAR(64) NOT NULL,
	"status" SMALLINT NOT NULL,
	"uid" INTEGER NOT NULL,
	FOREIGN KEY ("uid") REFERENCES "users"("id")
);

CREATE TABLE "achievements_users"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"game_id" INTEGER NOT NULL,
	"created_at" TIMESTAMP NOT NULL,
	"code" SMALLINT NOT NULL,
	FOREIGN KEY ("game_id") REFERENCES "game"("id")
);

