export type Status = 'up' | 'degraded' | 'down' | 'unknown'

export interface LatestCheck {
  id: string
  checked_at: string
  status: Status
  response_ms: number | null
  error_message: string | null
  detail: unknown
}

export interface System {
  id: string
  name: string
  description: string | null
  health: Status
  service_count: number
  created_at: string
  updated_at: string
}

export interface Service {
  id: string
  name: string
  service_type: string
  config: Record<string, unknown>
  interval_secs: number
  enabled: boolean
  system_ids: string[]
  created_at: string
  updated_at: string
  latest_check: LatestCheck | null
}

export interface CheckResult {
  id: string
  service_id: string
  checked_at: string
  status: Status
  response_ms: number | null
  detail: string | null
  error_message: string | null
}

export interface Incident {
  id: string
  service_id: string
  started_at: string
  resolved_at: string | null
  status: 'open' | 'resolved'
  trigger_status: string
  notes: string | null
}

export type WsMessage =
  | { type: 'check_completed'; service_id: string; check_id: string; checked_at: string; status: string; response_ms: number | null; detail: unknown; error_message: string | null }
  | { type: 'incident_opened'; incident_id: string; service_id: string; started_at: string; trigger_status: string }
  | { type: 'incident_resolved'; incident_id: string; service_id: string; resolved_at: string }
  | { type: 'service_updated'; service_id: string; fields: Record<string, unknown> }
  | { type: 'system_updated'; system_id: string; fields: Record<string, unknown> }
  | { type: 'ping'; ts: string }
