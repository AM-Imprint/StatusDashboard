<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useCheckResultsStore } from '../stores/checkResults'
import { useServicesStore } from '../stores/services'
import { checkStats } from '../utils/serviceDetail'
import type { CheckResult } from '../types'

const props = defineProps<{ serviceId: string }>()
const store = useCheckResultsStore()
const servicesStore = useServicesStore()

onMounted(() => store.fetchForService(props.serviceId))

const results = computed(() => store.byService[props.serviceId] ?? [])
const loading = computed(() => store.loading[props.serviceId])
const serviceType = computed(() => servicesStore.items[props.serviceId]?.service_type ?? '')

const fmtTime = (iso: string) =>
  new Date(iso).toLocaleString(undefined, {
    month: 'short', day: 'numeric',
    hour: '2-digit', minute: '2-digit', second: '2-digit',
  })

const fmtMs = (ms: number | null) =>
  ms == null ? null : ms >= 1000 ? `${(ms / 1000).toFixed(2)}s` : `${ms}ms`

function parseDetail(raw: unknown): Record<string, unknown> | null {
  if (!raw) return null
  if (typeof raw === 'string') { try { return JSON.parse(raw) } catch { return null } }
  if (typeof raw === 'object') return raw as Record<string, unknown>
  return null
}

function detailRows(r: CheckResult): { label: string; value: string }[] {
  return checkStats(serviceType.value, r.detail, r.response_ms)
}

function fmtDetail(raw: unknown): string | null {
  const d = parseDetail(raw)
  if (!d) return null
  return JSON.stringify(d, null, 2)
}
</script>

<template>
  <div class="check-list">
    <div v-if="loading && results.length === 0" class="loading">Loading…</div>
    <div v-else-if="results.length === 0" class="loading">No checks recorded yet.</div>

    <div v-for="r in results" :key="r.id" :class="['check-entry', r.status]">
      <!-- Header row -->
      <div class="check-entry-header">
        <span :class="['check-dot', r.status]"></span>
        <span class="check-status-text">{{ r.status.toUpperCase() }}</span>
        <span class="check-time">{{ fmtTime(r.checked_at) }}</span>
        <span v-if="fmtMs(r.response_ms)" class="check-ms">{{ fmtMs(r.response_ms) }}</span>
      </div>

      <!-- Error -->
      <div v-if="r.error_message" class="check-error">{{ r.error_message }}</div>

      <!-- Stats -->
      <div v-if="detailRows(r).length" class="check-detail-stats">
        <span v-for="s in detailRows(r)" :key="s.label" class="check-detail-stat">
          <span class="check-detail-label">{{ s.label }}</span>
          <span class="check-detail-value">{{ s.value }}</span>
        </span>
      </div>

      <!-- Raw detail JSON -->
      <pre v-if="fmtDetail(r.detail)" class="check-detail-json">{{ fmtDetail(r.detail) }}</pre>
    </div>
  </div>
</template>

<style scoped>
.check-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.check-entry {
  border: 1px solid var(--border);
  border-left: 3px solid transparent;
  border-radius: var(--radius-sm);
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  background: var(--bg);
}

.check-entry.up       { border-left-color: var(--color-up); }
.check-entry.degraded { border-left-color: var(--color-degraded); }
.check-entry.down     { border-left-color: var(--color-down); }

.check-entry-header {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.check-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.check-dot.up       { background: var(--color-up); }
.check-dot.degraded { background: var(--color-degraded); }
.check-dot.down     { background: var(--color-down); }
.check-dot.unknown  { background: var(--color-unknown); }

.check-status-text {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: var(--text-muted);
}

.check-time {
  font-size: 12px;
  color: var(--text-muted);
  flex: 1;
}

.check-ms {
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  color: var(--text-muted);
  font-family: 'SFMono-Regular', Consolas, monospace;
}

.check-error {
  font-size: 12px;
  color: var(--color-down);
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.5;
  background: color-mix(in srgb, var(--color-down) 8%, transparent);
  border-radius: var(--radius-sm);
  padding: 6px 8px;
}

.check-detail-stats {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.check-detail-stat {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.check-detail-label {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  color: var(--text-muted);
}

.check-detail-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  font-variant-numeric: tabular-nums;
  word-break: break-word;
}

.check-detail-json {
  font-size: 11px;
  font-family: 'SFMono-Regular', Consolas, monospace;
  color: var(--text-muted);
  background: var(--bg-hover);
  border-radius: var(--radius-sm);
  padding: 8px 10px;
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.6;
  margin: 0;
}
</style>
