import { reactive } from 'vue'
export const confirmation = reactive({ open: false, description: '' })
let resolveConfirmation: ((value: boolean) => void) | undefined
export function askConfirmation(description: string): Promise<boolean> {
  resolveConfirmation?.(false)
  confirmation.description = description
  confirmation.open = true
  return new Promise(resolve => { resolveConfirmation = resolve })
}
export function finishConfirmation(value: boolean) {
  confirmation.open = false
  resolveConfirmation?.(value)
  resolveConfirmation = undefined
}

export function dismissConfirmation() {
  queueMicrotask(() => { if (confirmation.open) finishConfirmation(false) })
}
