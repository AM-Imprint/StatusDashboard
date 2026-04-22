<script setup lang="ts">
import { computed } from 'vue'
import { useServicesStore } from '../stores/services'
import type { System, Service } from '../types'
import TvServiceSlice from './TvServiceSlice.vue'

const props = defineProps<{ system: System; span?: number }>()

const servicesStore = useServicesStore()
const services = computed(() => servicesStore.bySystem(props.system.id))

const effectiveSpan = computed(() => props.span ?? 1)

// Greedily distribute services into `span` buckets by visual weight
const subColumns = computed<Service[][]>(() => {
  const n = effectiveSpan.value
  if (n <= 1) return [services.value]

  const buckets: Service[][] = Array.from({ length: n }, () => [])
  const weights = new Array(n).fill(0)

  for (const svc of services.value) {
    const w = svc.service_type === 'chart_query' ? 3 : 1
    const lightest = weights.indexOf(Math.min(...weights))
    buckets[lightest].push(svc)
    weights[lightest] += w
  }

  return buckets
})
</script>

<template>
  <div
    class="tv-column"
    :class="system.health"
    :style="{ gridColumn: `span ${effectiveSpan}` }"
  >
    <div class="tv-column-header">
      <span class="tv-column-title">{{ system.name }}</span>
      <span class="tv-column-health">{{ system.health.toUpperCase() }}</span>
    </div>
    <div class="tv-column-body">
      <div
        v-for="(bucket, i) in subColumns"
        :key="i"
        class="tv-sub-column"
        :class="{ 'tv-sub-column--divided': i > 0 }"
      >
        <TvServiceSlice
          v-for="svc in bucket"
          :key="svc.id"
          :service="svc"
        />
        <div v-if="bucket.length === 0 && subColumns.length === 1" class="tv-no-services">No services</div>
      </div>
      <div v-if="services.length === 0" class="tv-no-services tv-no-services--full">No services</div>
    </div>
  </div>
</template>

<style scoped>
.tv-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  border-right: 3px solid rgba(0, 0, 0, 0.15);
  overflow: hidden;
}

.tv-column:last-child {
  border-right: none;
}

.tv-column-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 14px 10px 12px;
  border-bottom: 3px solid rgba(0, 0, 0, 0.2);
  flex-shrink: 0;
}

.tv-column.up      .tv-column-header { background: #15803d; }
.tv-column.degraded .tv-column-header { background: #b45309; }
.tv-column.down    .tv-column-header { background: #b91c1c; }
.tv-column.unknown  .tv-column-header { background: #374151; }

.tv-column-title {
  font-size: clamp(16px, 2vw, 32px);
  font-weight: 800;
  color: #000;
  text-align: center;
  line-height: 1.1;
  letter-spacing: -0.5px;
}

.tv-column-health {
  font-size: clamp(10px, 1.2vw, 18px);
  font-weight: 700;
  color: rgba(0, 0, 0, 0.6);
  letter-spacing: 1.5px;
  margin-top: 2px;
}

.tv-column-body {
  flex: 1;
  display: flex;
  flex-direction: row;
  min-height: 0;
}

.tv-sub-column {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
}

.tv-sub-column--divided {
  border-left: 2px solid rgba(0, 0, 0, 0.15);
}

.tv-no-services {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(0, 0, 0, 0.4);
  font-size: 14px;
}

.tv-no-services--full {
  background: #6b7280;
}
</style>
