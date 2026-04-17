<script setup lang="ts">
import { ref, computed } from 'vue'
import { useServicesStore } from './stores/services'
import { useSystemsStore } from './stores/systems'
import { useWebSocket } from './composables/useWebSocket'
import WsIndicator   from './components/WsIndicator.vue'
import ServiceGrid   from './components/ServiceGrid.vue'
import ServiceModal  from './components/ServiceModal.vue'
import ServiceDetail from './components/ServiceDetail.vue'
import SystemModal   from './components/SystemModal.vue'

const { status } = useWebSocket()
const servicesStore = useServicesStore()
const systemsStore  = useSystemsStore()

const showServiceModal = ref(false)
const editingServiceId = ref<string | null>(null)

const showSystemModal  = ref(false)
const editingSystemId  = ref<string | null>(null)

const selectedId = ref<string | null>(null)

const editingService = computed(() =>
  editingServiceId.value ? servicesStore.items[editingServiceId.value] : undefined
)
const editingSystem = computed(() =>
  editingSystemId.value ? systemsStore.items[editingSystemId.value] : undefined
)

function openAddService() {
  editingServiceId.value = null
  showServiceModal.value = true
}

function openEditService(id: string) {
  editingServiceId.value = id
  showServiceModal.value = true
}

function openAddSystem() {
  editingSystemId.value = null
  showSystemModal.value = true
}

function openEditSystem(id: string) {
  editingSystemId.value = id
  showSystemModal.value = true
}

function selectService(id: string) {
  selectedId.value = id === selectedId.value ? null : id
}
</script>

<template>
  <nav class="nav">
    <span class="nav-title">Status Dashboard</span>
    <WsIndicator :status="status" />
    <button class="btn" @click="openAddSystem">+ System</button>
    <button class="btn btn-primary" @click="openAddService">+ Service</button>
  </nav>

  <ServiceGrid
    @select="selectService"
    @editSystem="openEditSystem"
  />

  <ServiceDetail
    v-if="selectedId"
    :service-id="selectedId"
    @close="selectedId = null"
    @edit="openEditService"
  />

  <ServiceModal
    v-if="showServiceModal"
    :service="editingService"
    @close="showServiceModal = false"
  />

  <SystemModal
    v-if="showSystemModal"
    :system="editingSystem"
    @close="showSystemModal = false"
  />
</template>
