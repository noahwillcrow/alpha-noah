CREATE TABLE GameStateRecords (
    GameName TEXT NOT NULL,
    StateHash BLOB NOT NULL,
    DrawsCount INTEGER NOT NULL,
    LossesCount INTEGER NOT NULL,
    WinsCount INTEGER NOT NULL,
    PRIMARY KEY (GameName, StateHash)
);

CREATE TABLE GameLogs (
    ID INTEGER NOT NULL PRIMARY KEY,
    GameName TEXT NOT NULL,
    Log BLOB NOT NULL
);

CREATE INDEX IDX_GameLogs_GameName ON GameLogs(GameName);