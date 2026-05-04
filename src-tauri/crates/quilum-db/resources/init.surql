USE NAMESPACE quilum DATABASE main;
DEFINE INDEX IF NOT EXISTS idx_blocked_app_identifier ON blocked_app COLUMNS identifier UNIQUE;
