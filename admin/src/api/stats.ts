export function getServerStats() {
  return fetch('/api/stats')
  .then(a => a.json());
}
