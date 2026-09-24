import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import App from './App.vue'

const recipientId = '11111111-1111-4111-8111-111111111111'
const templates = ['ios_push', 'android_push', 'sms', 'email'].map((channel, index) => ({
  template_id: `22222222-2222-4222-8222-22222222222${index + 1}`,
  version: 1,
  name: 'Order update',
  channel,
  subject: channel === 'email' ? 'Order ready' : '',
  body: 'Your order {{order_id}} is ready.',
}))

function response(body: unknown, status = 200) {
  return Promise.resolve(new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } }))
}

describe('Notification Console', () => {
  let fetchMock: ReturnType<typeof vi.fn>
  let notifications: Array<Record<string, unknown>>

  beforeEach(() => {
    notifications = []
    fetchMock = vi.fn((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input)
      if (url.endsWith('/v1/recipients')) return response({ items: [{ id: recipientId, display_name: 'Alex Example', email_masked: 'a***@example.test' }] })
      if (url.endsWith('/v1/templates')) return response({ items: templates })
      if (url.includes('/preferences/')) return response({ recipient_id: recipientId, channel: url.split('/').at(-1), opt_in: true })
      if (url.includes('/v1/notifications') && init?.method === 'POST') {
        const payload = JSON.parse(String(init.body))
        const notification = { id: `sample-${notifications.length + 1}`, recipient_id: recipientId, channel: payload.channel, status: 'accepted', attempts: 0, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
        notifications.unshift(notification)
        return response({ notification_id: notification.id, status: 'accepted', created_at: notification.created_at }, 202)
      }
      if (url.includes('/v1/notifications')) return response({ items: notifications })
      return response({ error: 'unexpected request' }, 404)
    })
    vi.stubGlobal('fetch', fetchMock)
  })

  afterEach(() => vi.unstubAllGlobals())

  it('loads synthetic data and submits a channel request', async () => {
    const wrapper = mount(App)
    await flushPromises()
    expect(wrapper.text()).toContain('Alex Example')
    expect(wrapper.text()).toContain('Synthetic data only')
    await wrapper.get('#body-input').setValue('Synthetic welcome')
    await wrapper.get('form').trigger('submit')
    await flushPromises()
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/notifications', expect.objectContaining({ method: 'POST' }))
    expect(wrapper.text()).toContain('accepted')
    expect(wrapper.get('output.alert-success').element.tagName).toBe('OUTPUT')
    expect(wrapper.get('aside[aria-label="Notification controls"]').element.tagName).toBe('ASIDE')
  })

  it('shows validation feedback for invalid variable JSON', async () => {
    const wrapper = mount(App)
    await flushPromises()
    await wrapper.get('#variables-input').setValue('{')
    await wrapper.get('form').trigger('submit')
    await flushPromises()
    expect(wrapper.get('[role="alert"]').text()).toContain('valid JSON object')
    expect(fetchMock.mock.calls.some(([url]) => String(url).endsWith('/v1/notifications'))).toBe(false)
  })
})
