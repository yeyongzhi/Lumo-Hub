import { customRef } from 'vue';
import { toast } from 'vue-sonner';

/** A message that remains readable by its owner while displaying as a toast. */
export function useToastMessage() {
  let value = '';
  return customRef<string>((track, trigger) => ({
    get() {
      track();
      return value;
    },
    set(next) {
      value = next;
      if (next) toast(next);
      trigger();
    },
  }));
}
