import type { Channel, Notification, Preference, Recipient, Template } from './types'

const API_KEY = import.meta.env.VITE_API_KEY || 'local-dev-api-key'
const API_BASE = import.meta.env.VITE_API_BASE_URL || '/api'

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: {
      'content-type': 'application/json',
      'x-api-key': API_KEY,
      ...init.headers,
    },
  })
  const payload = await response.json().catch(() => ({}))
  if (!response.ok) throw new Error(payload.error || `Request failed (${response.status})`)
  return payload as T
}

export const api = {
  recipients: () => request<{ items: Recipient[] }>('/v1/recipients'),
  templates: () => request<{ items: Template[] }>('/v1/templates'),
  notifications: (recipientId?: string) => {
    const query = recipientId ? `?recipient_id=${encodeURIComponent(recipientId)}&limit=50` : '?limit=50'
    return request<{ items: Notification[] }>(`/v1/notifications${query}`)
  },
  preference: (recipientId: string, channel: Channel) =>
    request<Preference>(`/v1/recipients/${recipientId}/preferences/${channel}`),
  setPreference: (recipientId: string, channel: Channel, optIn: boolean) =>
    request<Preference>(`/v1/recipients/${recipientId}/preferences/${channel}`, {
      method: 'PUT', body: JSON.stringify({ opt_in: optIn }),
    }),
  createNotification: (payload: Record<string, unknown>, idempotencyKey: string) =>
    request<{ notification_id: string; status: string; created_at: string }>('/v1/notifications', {
      method: 'POST',
      headers: { 'idempotency-key': idempotencyKey },
      body: JSON.stringify(payload),
    }),
}
