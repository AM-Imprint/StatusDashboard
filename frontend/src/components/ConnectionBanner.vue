<script setup lang="ts">
defineProps<{
  status: 'connecting' | 'open' | 'closed' | 'error'
  retryIn: number | null
}>()
defineEmits<{ reconnect: [] }>()
</script>

<template>
  <Transition name="banner">
    <div v-if="status === 'closed' || status === 'error'" class="conn-banner">
      <div class="conn-banner-inner">
        <span class="conn-banner-icon">⚠</span>
        <span class="conn-banner-msg">
          {{ status === 'error' ? 'Connection error — cannot reach the backend.' : 'Connection lost — backend is unreachable.' }}
        </span>
        <span v-if="retryIn !== null" class="conn-banner-retry-info">
          Retrying in {{ retryIn }}s
        </span>
        <button class="conn-banner-btn" @click="$emit('reconnect')">Retry now</button>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.conn-banner {
  position: sticky;
  top: var(--nav-h);
  z-index: 90;
  width: 100%;
  background: #b91c1c;
  color: #fff;
  font-weight: 600;
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
}

.conn-banner-inner {
  display: flex;
  align-items: center;
  gap: 10px;
  max-width: 1200px;
  margin: 0 auto;
  padding: 10px 16px;
}

.conn-banner-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.conn-banner-msg {
  flex: 1;
  font-size: 14px;
}

.conn-banner-retry-info {
  font-size: 13px;
  opacity: 0.85;
  white-space: nowrap;
}

.conn-banner-btn {
  background: rgba(255,255,255,0.2);
  border: 1px solid rgba(255,255,255,0.5);
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  padding: 4px 12px;
  border-radius: 4px;
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.15s;
}

.conn-banner-btn:hover {
  background: rgba(255,255,255,0.35);
}

.banner-enter-active,
.banner-leave-active {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.banner-enter-from,
.banner-leave-to {
  transform: translateY(-100%);
  opacity: 0;
}
</style>
