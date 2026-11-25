export const updateIntervals = [120, 360, 1440, 4320, 10080, 43200] as const;

export function autoUpdateEnabled(client: Record<string, unknown> | undefined) {
  return client?.['lumo-auto-update-check'] !== false;
}

export function updateInterval(client: Record<string, unknown> | undefined) {
  const value = client?.['lumo-update-check-interval'];
  return typeof value === 'number' && updateIntervals.some((interval) => interval === value)
    ? value
    : 1440;
}

export function autoDownloadEnabled(client: Record<string, unknown> | undefined) {
  return client?.['lumo-auto-update-download'] !== false;
}
