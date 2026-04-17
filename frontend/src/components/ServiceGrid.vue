<script setup lang="ts">
import { computed } from 'vue'
import { useServicesStore } from '../stores/services'
import { useSystemsStore } from '../stores/systems'
import SystemCard from './SystemCard.vue'
import ServiceCard from './ServiceCard.vue'

const emit = defineEmits<{
  select: [id: string]
  editSystem: [id: string]
}>()

const servicesStore = useServicesStore()
const systemsStore  = useSystemsStore()

const systems   = computed(() => systemsStore.list)
const ungrouped = computed(() => servicesStore.ungrouped)

const isEmpty = computed(() => systems.value.length === 0 && ungrouped.value.length === 0)
</script>

<template>
  <div style="padding: 20px; display: flex; flex-direction: column; gap: 20px;">
    <template v-if="servicesStore.loading && isEmpty">
      <div class="loading">Loading…</div>
    </template>
    <template v-else-if="servicesStore.error">
      <div class="error-msg">{{ servicesStore.error }}</div>
    </template>
    <template v-else-if="isEmpty">
      <div class="empty-state">
        <p>No systems or services configured.</p>
        <p style="font-size: 13px">Create a <strong>System</strong> to group your checks, or add a standalone service.</p>
      </div>
    </template>

    <!-- Systems grid -->
    <div v-if="systems.length > 0" class="service-grid" style="padding: 0">
      <SystemCard
        v-for="sys in systems"
        :key="sys.id"
        :system="sys"
        @selectService="emit('select', $event)"
        @editSystem="emit('editSystem', $event)"
      />
    </div>

    <!-- Ungrouped services -->
    <template v-if="ungrouped.length > 0">
      <div class="section-header">Ungrouped</div>
      <div class="service-grid" style="padding: 0">
        <ServiceCard
          v-for="svc in ungrouped"
          :key="svc.id"
          :service="svc"
          @select="emit('select', $event)"
        />
      </div>
    </template>
  </div>
</template>
