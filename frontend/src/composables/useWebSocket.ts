import { ref, onUnmounted } from 'vue'
import { useServicesStore } from '../stores/services'
import { useCheckResultsStore } from '../stores/checkResults'
import { useIncidentsStore } from '../stores/incidents'
import { useSystemsStore } from '../stores/systems'
import type { WsMessage } from '../types'

type WsStatus = 'connecting' | 'open' | 'closed' | 'error'

const WS_URL = (import.meta.env.VITE_WS_URL as string | undefined) ?? '/ws'
const MAX_BACKOFF = 30_000

export function useWebSocket() {
  const status = ref<WsStatus>('connecting')
  const retryIn = ref<number | null>(null)
  let ws: WebSocket | null = null
  let retryTimeout: ReturnType<typeof setTimeout> | null = null
  let countdownInterval: ReturnType<typeof setInterval> | null = null
  let backoff = 1000
  let destroyed = false

  const servicesStore  = useServicesStore()
  const checkStore     = useCheckResultsStore()
  const incidentsStore = useIncidentsStore()
  const systemsStore   = useSystemsStore()

  function dispatch(msg: WsMessage) {
    switch (msg.type) {
      case 'check_completed':
        servicesStore.applyCheckUpdate(msg)
        checkStore.prepend(msg.service_id, msg)
        break
      case 'incident_opened':
        incidentsStore.applyOpened(msg)
        break
      case 'incident_resolved':
        incidentsStore.applyResolved(msg)
        break
      case 'service_updated':
        servicesStore.applyServiceUpdate(msg)
        break
      case 'system_updated':
        systemsStore.applySystemUpdate(msg)
        break
      case 'ping':
        break
    }
  }

  function connect() {
    if (destroyed) return
    status.value = 'connecting'
    ws = new WebSocket(WS_URL)

    ws.onopen = () => {
      backoff = 1000
      status.value = 'open'
      retryIn.value = null
      if (countdownInterval) { clearInterval(countdownInterval); countdownInterval = null }
      systemsStore.fetchAll()
      servicesStore.fetchAll()
    }

    ws.onmessage = (event) => {
      try {
        dispatch(JSON.parse(event.data as string) as WsMessage)
      } catch {
        // ignore malformed messages
      }
    }

    ws.onerror = () => {
      status.value = 'error'
    }

    ws.onclose = () => {
      if (destroyed) return
      status.value = 'closed'
      const delay = backoff
      retryIn.value = Math.ceil(delay / 1000)
      if (countdownInterval) clearInterval(countdownInterval)
      countdownInterval = setInterval(() => {
        if (retryIn.value !== null && retryIn.value > 1) {
          retryIn.value--
        } else {
          clearInterval(countdownInterval!)
          countdownInterval = null
        }
      }, 1000)
      retryTimeout = setTimeout(() => {
        backoff = Math.min(backoff * 2, MAX_BACKOFF)
        connect()
      }, delay)
    }
  }

  function reconnectNow() {
    if (retryTimeout) { clearTimeout(retryTimeout); retryTimeout = null }
    if (countdownInterval) { clearInterval(countdownInterval); countdownInterval = null }
    retryIn.value = null
    backoff = 1000
    connect()
  }

  function disconnect() {
    destroyed = true
    if (retryTimeout) clearTimeout(retryTimeout)
    if (countdownInterval) clearInterval(countdownInterval)
    ws?.close()
    ws = null
  }

  connect()
  onUnmounted(disconnect)

  return { status, retryIn, connect, reconnectNow, disconnect }
}
