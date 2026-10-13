import { onMounted, onUnmounted, ref } from 'vue'

/** Refresh after midnight, waking from sleep, or returning to this window. */
export function useCurrentDay() {
  const now = ref(new Date())
  const refresh = () => { now.value = new Date() }
  let timer: ReturnType<typeof setInterval>
  onMounted(() => {
    timer = setInterval(refresh, 30_000)
    window.addEventListener('focus', refresh)
    document.addEventListener('visibilitychange', refresh)
  })
  onUnmounted(() => {
    clearInterval(timer)
    window.removeEventListener('focus', refresh)
    document.removeEventListener('visibilitychange', refresh)
  })
  return now
}
