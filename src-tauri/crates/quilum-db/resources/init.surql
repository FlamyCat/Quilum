USE NAMESPACE quilum DATABASE main;

DEFINE INDEX IF NOT EXISTS idx_blocked_app_identifier ON blocked_app COLUMNS identifier UNIQUE;

DEFINE TABLE IF NOT EXISTS events SCHEMAFULL TYPE NORMAL;
DEFINE FIELD name ON events TYPE string ASSERT $value != "";
DEFINE FIELD description ON events TYPE string;
DEFINE FIELD starts_at ON events TYPE int;
DEFINE FIELD ends_at ON events TYPE int;

DEFINE TABLE IF NOT EXISTS slots SCHEMAFULL TYPE NORMAL;
DEFINE FIELD starts_at ON slots TYPE int;
DEFINE FIELD ends_at ON slots TYPE int;

DEFINE TABLE IF NOT EXISTS tasks SCHEMAFULL TYPE NORMAL;
DEFINE FIELD name ON tasks TYPE string ASSERT $value != "";
DEFINE FIELD description ON tasks TYPE string;
DEFINE FIELD priority ON tasks TYPE string;
DEFINE FIELD estimated_duration ON tasks TYPE int;
DEFINE FIELD deadline ON tasks TYPE int;
DEFINE FIELD completed ON tasks TYPE bool;
DEFINE FIELD scheduled_for ON tasks TYPE option<int>;

DEFINE TABLE IF NOT EXISTS task_lists SCHEMAFULL TYPE NORMAL;
DEFINE FIELD name ON task_lists TYPE string ASSERT $value != "";
DEFINE FIELD description ON task_lists TYPE string;

DEFINE TABLE IF NOT EXISTS blocked_apps SCHEMAFULL TYPE NORMAL;
DEFINE FIELD identifier ON blocked_apps TYPE string;
DEFINE FIELD display_name ON blocked_apps TYPE string;
DEFINE FIELD blocked ON blocked_apps TYPE bool;

DEFINE TABLE IF NOT EXISTS contains SCHEMAFULL TYPE RELATION FROM task_lists TO tasks ENFORCED;
DEFINE TABLE IF NOT EXISTS scheduled_in SCHEMAFULL TYPE RELATION FROM tasks TO slots ENFORCED;