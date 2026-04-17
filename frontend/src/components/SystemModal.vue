<script setup lang="ts">
import { ref, computed } from 'vue'
import { api } from '../api/http'
import { useSystemsStore } from '../stores/systems'
import { useServicesStore } from '../stores/services'
import type { System } from '../types'

const props = defineProps<{ system?: System }>()
const emit  = defineEmits<{ close: [] }>()

const store         = useSystemsStore()
const servicesStore = useServicesStore()
const isEdit        = computed(() => !!props.system)

const name        = ref(props.system?.name ?? '')
const description = ref(props.system?.description ?? '')
const error       = ref('')
const saving      = ref(false)
const deleting    = ref(false)

async function submit() {
  error.value = ''
  saving.value = true
  try {
    if (isEdit.value && props.system) {
      const updated = await api.updateSystem(props.system.id, {
        name: name.value,
        description: description.value || null,
      })
      store.upsert({ ...props.system, name: updated.name, updated_at: updated.updated_at })
    } else {
      const created = await api.createSystem({
        name: name.value,
        description: description.value || null,
      })
      store.upsert(created)
    }
    emit('close')
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Save failed'
  } finally {
    saving.value = false
  }
}

async function deleteSystem() {
  if (!props.system) return
  const affected = servicesStore.bySystem(props.system.id).length
  const msg = affected > 0
    ? `Delete "${props.system.name}"? Its ${affected} service(s) will become ungrouped.`
    : `Delete "${props.system.name}"?`
  if (!confirm(msg)) return

  deleting.value = true
  try {
    await api.deleteSystem(props.system.id)
    store.remove(props.system.id)
    // Services become ungrouped — refetch to get updated system_id = null
    await servicesStore.fetchAll()
    emit('close')
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Delete failed'
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal">
      <div class="modal-header">
        <span class="modal-title">{{ isEdit ? 'Edit System' : 'New System' }}</span>
        <button class="icon-btn" @click="emit('close')">✕</button>
      </div>

      <form class="modal-body" @submit.prevent="submit">
        <div class="form-group">
          <label for="sys-name">Name</label>
          <input id="sys-name" type="text" v-model="name" placeholder="Production App" required />
        </div>

        <div class="form-group">
          <label for="sys-desc">Description</label>
          <input id="sys-desc" type="text" v-model="description" placeholder="Optional description" />
        </div>

        <div v-if="error" class="form-error">{{ error }}</div>
      </form>

      <div class="modal-footer">
        <div>
          <button
            v-if="isEdit"
            type="button"
            class="btn btn-danger btn-sm"
            :disabled="deleting"
            @click="deleteSystem"
          >{{ deleting ? 'Deleting…' : 'Delete System' }}</button>
        </div>
        <div class="modal-footer-right">
          <button type="button" class="btn" @click="emit('close')">Cancel</button>
          <button type="submit" class="btn btn-primary" :disabled="saving" @click="submit">
            {{ saving ? 'Saving…' : (isEdit ? 'Save Changes' : 'Create System') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
