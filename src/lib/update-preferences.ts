export const updateIntervals = [10, 30, 60, 360, 1440] as const;

export function autoUpdateEnabled(client: Record<string, unknown> | undefined) {
  return client?.['lumo-auto-update-check'] !== false;
}

export function updateInterval(client: Record<string, unknown> | undefined) {
  const value = client?.['lumo-update-check-interval'];
  return typeof value === 'number' && updateIntervals.some((interval) => interval === value)
    ? value
    : 10;
}
