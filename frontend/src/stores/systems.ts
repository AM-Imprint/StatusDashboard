import { defineStore } from 'pinia'
import { api } from '../api/http'
import type { Status, System, WsMessage } from '../types'
import { useServicesStore } from './services'

export const useSystemsStore = defineStore('systems', {
  state: () => ({
    items: {} as Record<string, System>,
    loading: false,
    error: null as string | null,
  }),

  getters: {
    list: (state) => Object.values(state.items).sort((a, b) => a.created_at.localeCompare(b.created_at)),
  },

  actions: {
    async fetchAll() {
      this.loading = true
      this.error = null
      try {
        const systems = await api.fetchSystems()
        const map: Record<string, System> = {}
        for (const s of systems) map[s.id] = s
        this.items = map
      } catch (e) {
        this.error = e instanceof Error ? e.message : 'Failed to load systems'
      } finally {
        this.loading = false
      }
    },

    upsert(system: System) {
      this.items[system.id] = system
    },

    remove(id: string) {
      delete this.items[id]
    },

    applySystemUpdate(msg: Extract<WsMessage, { type: 'system_updated' }>) {
      const sys = this.items[msg.system_id]
      if (!sys) return
      Object.assign(sys, msg.fields)
    },

    recomputeHealth(systemId: string) {
      const sys = this.items[systemId]
      if (!sys) return

      const servicesStore = useServicesStore()
      const services = servicesStore.list.filter(s => s.system_ids.includes(systemId))

      if (services.length === 0) {
        sys.health = 'unknown'
        sys.service_count = 0
        return
      }

      sys.service_count = services.length
      let worst: Status = 'unknown'
      for (const svc of services) {
        const s = svc.latest_check?.status
        if (s === 'down')              { worst = 'down'; break }
        if (s === 'degraded')          { worst = 'degraded' }
        if (s === 'up' && worst !== 'degraded') { worst = 'up' }
      }
      sys.health = worst
    },
  },
})
