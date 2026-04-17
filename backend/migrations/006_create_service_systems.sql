CREATE TABLE IF NOT EXISTS service_systems (
    service_id TEXT NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    system_id  TEXT NOT NULL REFERENCES systems(id) ON DELETE CASCADE,
    PRIMARY KEY (service_id, system_id)
);

-- Migrate existing single-system assignments into the join table
INSERT OR IGNORE INTO service_systems (service_id, system_id)
SELECT id, system_id FROM services WHERE system_id IS NOT NULL;
