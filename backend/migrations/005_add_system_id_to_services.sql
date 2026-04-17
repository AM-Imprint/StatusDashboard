ALTER TABLE services ADD COLUMN system_id TEXT REFERENCES systems(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_services_system ON services(system_id);
