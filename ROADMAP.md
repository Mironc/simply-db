# What is this? 

This file shows implemented, planned and not planned features.

Not planned features are marked with ~~strikethrough~~

If feature is marked with Out-Of-Scope that means it likely won't be ever implemented.

# Core

## Parser

- [x] UTF-8 compatibility.
- [x] Lazy token pulling.
- [x] Recursive-Descent expression parsing.

## Storage
- [x] Table level synchronization (RwLock).
- [x] Flat/non-hierarchical storage.
- [ ] Indexes. Likely hashmap.
- [ ] ~~Raw byte representation~~.
- [ ] ~~MVCC synchronization~~.

## Data Types

- [x] BOOLEAN - bool.
- [x] TEXT - String.
- [x] INT - i32.
- [x] FLOAT - f32.
- [ ] ~~Tuples/Structures~~. Out-Of-Scope.

## Field Modifiers

- [x] NOT NULL.
- [x] UNIQUE.
- [x] PRIMARY KEY. But it isn't enforced to be unique in the table schema.
- [ ] CHECK.
- [ ] DEFAULT.
- [ ] AUTOINCREMENT.
- [ ] FOREIGN KEY.

## Expressions

They are used in projections, set statements and filtering.

- [x] Basic operations with data types. Examples: `1 + 2`, `name < 'B'`, `is_active AND is_admin` 
- [ ] Functions. Example: `UPPER('apple')`
- [ ] SQL statements. `IF`, `CASE`, `IN`, etc. Example: `IF price > 100 THEN ... ELSE ...`

## CREATE TABLE

- [x] Simple. Example: `CREATE TABLE users (id INT, name TEXT)`.
- [x] IF NOT EXISTS flag. `CREATE TABLE users IF NOT EXISTS (id INT, name TEXT)`.
- [x] [Field Modifiers](#Field-Modifiers). `CREATE TABLE users (id INT PRIMARY KEY, name TEXT NOT NULL)`.

## SELECT

- [x] Simple. Example: `SELECT * FROM users`.
- [x] Filtering. Example: `SELECT * FROM users WHERE name == 'John'`.
- [x] Projection. Example: `SELECT id, name FROM users`.
- [x] Pagination. Example: `SELECT * FROM users SKIP 10 TAKE 5`.
- [ ] Ordering. Example: `SELECT * FROM users ORDER BY id DESC`.
- [ ] ~~Aggregation functions~~. Example `SELECT COUNT(*) FROM users`.
- [ ] ~~Grouping~~. Example `SELECT country, COUNT(*) FROM users GROUP BY сountry`.
- [ ] ~~Sub-queries~~. Out-Of-Scope.
- [ ] ~~JOINs~~. Out-Of-Scope.

## INSERT

- [x] Simple. Example: `INSERT INTO users (id, name) VALUES (0,'John')`.
- [x] Batching. Example: `INSERT INTO users (id, name) VALUES (0,'John'), (1,'Steve')`.
- [ ] ~~Add entries into indexes~~.

## UPDATE

- [x] Simple. Example: `UPDATE users SET age=age+1`.
- [x] Filtering. Example: `UPDATE users SET email=NULL WHERE id == 0`.
- [x] Unique constraint check after update.
- [ ] ~~Update affected indexes~~.

## DROP

- [x] Simple. Example: `DROP TABLE users`.
- [ ] ~~FOREIGN KEY constraint check~~. Restrict.

## DELETE

- [x] Simple. Example: `DELETE FROM users WHERE age < 18`.
- [ ] ~~Remove affected entries from indexes~~.
- [ ] ~~FOREIGN KEY constraint check~~. Depending on ON DELETE tag: RESTRICT, SET NULL or CASCADE.


## TRUNCATE

- [x] Simple. Example: `TRUNCATE TABLE users`.
- [ ] ~~Clear indexes~~.
- [ ] ~~FOREIGN KEY constraint check~~. Restrict.
- [ ] ~~AUTOINCREMENT counter reset~~.

# Client

## Features

- [x] Graphical UI.
- [x] Opening multiple connections.
- [x] Viewing table structure.
- [x] Viewing table rows.
- [x] Sending SQL queries.
- [ ] Tracking database health/online.
- [ ] ~~Viewing metrics~~.

# HTTP Server

## Routes

- [x] GET `/ping`.
- [x] GET `/health`.
- [ ] ~~GET `/metrics`~~.
- [x] POST `/v1/query`.
- [x] GET `/v1/overview`.

## GET `/health`
- [x] Allocation size in KiB.
- [x] Local time in rfc3339.
- [ ] Tracking state. Hardcoded with permanent state `Healthy`.

## Metrics

- [ ] ~~ Query latency. Read/Write (ms)~~.
- [ ] ~~Opened/Closed connections~~.
- [ ] ~~Active connections~~.
