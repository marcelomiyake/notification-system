import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from './api'

describe('notification API client', () => {
  afterEach(() => vi.unstubAllGlobals())

  it('sends the local caller key and encodes recipient list filters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ items: [] }), { status: 200 }),
    )
    vi.stubGlobal('fetch', fetchMock)

    await api.notifications('recipient with spaces')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/v1/notifications?recipient_id=recipient%20with%20spaces&limit=50',
      expect.objectContaining({
        headers: expect.objectContaining({
          'content-type': 'application/json',
          'x-api-key': 'local-dev-api-key',
        }),
      }),
    )
  })

  it('sends preference changes as JSON', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ recipient_id: 'recipient-1', channel: 'email', opt_in: false }), {
        status: 200,
      }),
    )
    vi.stubGlobal('fetch', fetchMock)

    await api.setPreference('recipient-1', 'email', false)

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/v1/recipients/recipient-1/preferences/email',
      expect.objectContaining({ method: 'PUT', body: JSON.stringify({ opt_in: false }) }),
    )
  })

  it('preserves API error messages and falls back when an error body is not JSON', async () => {
    const fetchMock = vi.fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({ error: 'caller is disabled' }), { status: 401 }))
      .mockResolvedValueOnce(new Response('not-json', { status: 502 }))
    vi.stubGlobal('fetch', fetchMock)

    await expect(api.recipients()).rejects.toThrow('caller is disabled')
    await expect(api.templates()).rejects.toThrow('Request failed (502)')
  })

  it('adds the idempotency key to notification submissions', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ notification_id: 'notification-1', status: 'accepted', created_at: '2026-09-24T12:00:00Z' }), {
        status: 202,
      }),
    )
    vi.stubGlobal('fetch', fetchMock)

    await api.createNotification({ channel: 'email' }, 'request-key-1')

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/v1/notifications',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({ 'idempotency-key': 'request-key-1' }),
      }),
    )
  })
})
