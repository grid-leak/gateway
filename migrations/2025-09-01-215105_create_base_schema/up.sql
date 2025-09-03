-- Your SQL goes here
CREATE TABLE "users"(
	"id" BIGINT NOT NULL PRIMARY KEY,
	"name" TEXT NOT NULL,
	"division_name" SMALLINT NOT NULL,
	"division_rank" SMALLINT NOT NULL
);

CREATE TABLE "challenge_bookmarks"(
	"user_id" BIGINT NOT NULL,
	"challenge_id" UUID NOT NULL,
	"bookmark_time" TIMESTAMP NOT NULL,
	PRIMARY KEY("user_id", "challenge_id"),
	FOREIGN KEY ("user_id") REFERENCES "users"("id")
);

CREATE TABLE "ugcs"(
	"id" UUID NOT NULL PRIMARY KEY,
	"user_id" BIGINT NOT NULL,
	"name" TEXT NOT NULL,
	"created_at" TIMESTAMP NOT NULL,
	"updated_at" TIMESTAMP NOT NULL,
	"published" BOOL NOT NULL,
	"type_id" SMALLINT NOT NULL,
	"transform" JSONB NOT NULL,
	"map_position" JSONB,
	"teleport_transform" JSONB,
	FOREIGN KEY ("user_id") REFERENCES "users"("id")
);

CREATE TABLE "user_stats"(
	"user_id" BIGINT NOT NULL PRIMARY KEY,
	"stats" JSONB NOT NULL,
	FOREIGN KEY ("user_id") REFERENCES "users"("id")
);

CREATE TABLE "ugc_bookmarks"(
	"user_id" BIGINT NOT NULL,
	"ugc_id" UUID NOT NULL,
	"bookmark_time" TIMESTAMP NOT NULL,
	PRIMARY KEY("user_id", "ugc_id"),
	FOREIGN KEY ("user_id") REFERENCES "users"("id"),
	FOREIGN KEY ("ugc_id") REFERENCES "ugcs"("id")
);

CREATE TABLE "kit_unlocks"(
	"user_id" BIGINT NOT NULL,
	"kit_id" UUID NOT NULL,
	"opened" BOOL NOT NULL,
	PRIMARY KEY("user_id", "kit_id"),
	FOREIGN KEY ("user_id") REFERENCES "users"("id")
);

