import { expect, test } from '@playwright/test'
import { randomUUID } from 'node:crypto'

const recipientId = '11111111-1111-4111-8111-111111111111'
const fixtureTemplates = ['ios_push', 'android_push', 'sms', 'email'].map((channel, index) => ({
  template_id: `22222222-2222-4222-8222-22222222222${index + 1}`,
  version: 1,
  name: 'Order update',
  channel,
  subject: channel === 'email' ? 'Order ready' : '',
  body: 'Your order {{order_id}} is ready.',
}))

test('operator workflows cover channels, scheduling, preferences, retries and duplicate requests', async ({ page, request }) => {
  if (process.env.E2E_BASE_URL) {
    const apiUrl = `${process.env.E2E_BASE_URL}/api/v1`
    const headers = { 'x-api-key': 'local-dev-api-key', 'content-type': 'application/json' }
    const recipient = recipientId
    const detail = async (id: string) => request.get(`${apiUrl}/notifications/${id}`, { headers })
    const resetSmsPreference = await request.put(`${apiUrl}/recipients/${recipient}/preferences/sms`, { headers, data: { opt_in: true } })
    expect(resetSmsPreference.status()).toBe(200)
    const waitFor = async (id: string, status: string, attempts?: number) => {
      await expect.poll(async () => {
        const response = await detail(id)
        if (!response.ok()) return `http-${response.status()}`
        const body = await response.json()
        const notification = body.notification
        return attempts === undefined ? notification.status : `${notification.status}:${notification.attempts}`
      }, { timeout: 25_000, intervals: [250, 500, 1000] }).toBe(attempts === undefined ? status : `${status}:${attempts}`)
    }
    const submitFromConsole = async (channel: string, body: string, simulation = 'success', scheduled = false) => {
      await page.getByLabel('Delivery channel').selectOption(channel)
      await page.getByLabel('Message body').fill(body)
      await page.getByLabel('Template variables').fill('{}')
      const simulationDetails = page.locator('details.simulation-details')
      if (!(await simulationDetails.evaluate(element => (element as HTMLDetailsElement).open))) await simulationDetails.locator('summary').click()
      await page.getByLabel('Simulated provider response').selectOption(simulation)
      const scheduleToggle = page.locator('.schedule-row input[type="checkbox"]')
      if ((await scheduleToggle.isChecked()) !== scheduled) await page.getByText('Schedule for later', { exact: true }).click()
      if (scheduled) {
        const target = new Date(Date.now() + 30 * 60_000)
        const local = new Date(target.getTime() - target.getTimezoneOffset() * 60_000).toISOString().slice(0, 16)
        await page.locator('#scheduled-at').fill(local)
      }
      const submitted = page.waitForResponse(response => response.url().endsWith('/api/v1/notifications') && response.request().method() === 'POST')
      await page.getByRole('button', { name: scheduled ? 'Schedule notification' : 'Send notification' }).click()
      const response = await submitted
      expect(response.status()).toBe(202)
      const accepted = await response.json()
      await expect(page.getByRole('status')).toContainText('accepted')
      return accepted.notification_id as string
    }

    await page.goto('/')
    await expect(page.getByRole('heading', { name: 'Send a notification' })).toBeVisible()
    for (const channel of ['ios_push', 'android_push', 'sms', 'email']) {
      const id = await submitFromConsole(channel, `Synthetic ${channel} E2E message`)
      await waitFor(id, 'sent')
    }

    const scheduledId = await submitFromConsole('email', 'Synthetic scheduled message', 'success', true)
    await waitFor(scheduledId, 'scheduled')

    const smsOptIn = page.getByLabel('SMS opt in')
    await smsOptIn.uncheck()
    await expect(page.getByRole('status')).toContainText('paused')
    await page.getByLabel('Delivery channel').selectOption('sms')
    await page.getByLabel('Message body').fill('Must be rejected after opt-out')
    await page.getByLabel('Template variables').fill('{}')
    const scheduleToggle = page.locator('.schedule-row input[type="checkbox"]')
    if (await scheduleToggle.isChecked()) await page.getByText('Schedule for later', { exact: true }).click()
    await page.getByRole('button', { name: 'Send notification' }).click()
    await expect(page.getByRole('alert')).toContainText('opted out')
    await smsOptIn.check()
    await expect(page.getByRole('status')).toContainText('enabled')

    const retryId = await submitFromConsole('email', 'Synthetic retry test', 'transient_failure')
    await waitFor(retryId, 'sent', 2)
    const permanentId = await submitFromConsole('email', 'Synthetic terminal failure test', 'permanent_failure')
    await waitFor(permanentId, 'failed', 1)

    const idempotencyKey = `e2e-${randomUUID()}`
    const payload = { recipient_id: recipient, channel: 'email', subject: 'Idempotency check', body: 'Synthetic duplicate request', variables: {} }
    const first = await request.post(`${apiUrl}/notifications`, { headers: { ...headers, 'idempotency-key': idempotencyKey }, data: payload })
    const duplicate = await request.post(`${apiUrl}/notifications`, { headers: { ...headers, 'idempotency-key': idempotencyKey }, data: payload })
    expect(first.status()).toBe(202)
    expect(duplicate.status()).toBe(202)
    expect((await duplicate.json()).notification_id).toBe((await first.json()).notification_id)
    const conflict = await request.post(`${apiUrl}/notifications`, { headers: { ...headers, 'idempotency-key': idempotencyKey }, data: { ...payload, body: 'Changed payload' } })
    expect(conflict.status()).toBe(409)
    return
  }

  const accepted: Array<Record<string, unknown>> = []
  await page.route('**/api/v1/**', async route => {
    const url = route.request().url()
    if (url.endsWith('/recipients')) return route.fulfill({ json: { items: [{ id: recipientId, display_name: 'Alex Example', email_masked: 'a***@example.test' }] } })
    if (url.endsWith('/templates')) return route.fulfill({ json: { items: fixtureTemplates } })
    if (url.includes('/preferences/')) return route.fulfill({ json: { recipient_id: recipientId, channel: 'email', opt_in: true } })
    if (route.request().method() === 'POST') {
      const body = route.request().postDataJSON()
      const notification = { id: 'e2e-1', recipient_id: recipientId, channel: body.channel, status: 'accepted', attempts: 0, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
      accepted.unshift(notification)
      return route.fulfill({ status: 202, json: { notification_id: notification.id, status: 'accepted', created_at: notification.created_at } })
    }
    return route.fulfill({ json: { items: accepted } })
  })

  await page.goto('/')
  await expect(page.getByRole('heading', { name: 'Send a notification' })).toBeVisible()
  await page.getByLabel('Delivery channel').selectOption('email')
  await page.getByLabel('Message body').fill('A mocked synthetic UI message')
  await page.getByLabel('Template variables').fill('{}')
  await page.getByRole('button', { name: 'Send notification' }).click()
  await expect(page.getByRole('status')).toContainText('accepted')
  await expect(page.locator('.activity-item').first()).toContainText('Email notification')
})
