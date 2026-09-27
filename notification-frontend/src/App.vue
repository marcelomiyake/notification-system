<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { api } from './api'
import { channels, type Channel, type Notification, type Recipient, type Template } from './types'

const channelLabels: Record<Channel, string> = {
  ios_push: 'iOS push', android_push: 'Android push', sms: 'SMS', email: 'Email',
}
const recipients = ref<Recipient[]>([])
const templates = ref<Template[]>([])
const notifications = ref<Notification[]>([])
const preferences = ref<Record<Channel, boolean>>({ ios_push: true, android_push: true, sms: true, email: true })
const selectedRecipient = ref('')
const selectedChannel = ref<Channel>('email')
const selectedTemplate = ref('')
const simulation = ref('success')
const subject = ref('')
const body = ref('')
const variablesText = ref('{\n  "order_id": "A-1042"\n}')
const scheduleEnabled = ref(false)
const scheduledAt = ref('')
const notice = ref('')
const errorMessage = ref('')
const loading = ref(true)
const submitting = ref(false)
const preferenceSaving = ref(false)
const activeView = ref('overview')

const channelTemplates = computed(() => templates.value.filter(template => template.channel === selectedChannel.value))
const selectedName = computed(() => recipients.value.find(person => person.id === selectedRecipient.value)?.display_name || 'Choose a recipient')
const recentNotifications = computed(() => notifications.value.slice(0, 6))
const counts = computed(() => ({
  total: notifications.value.length,
  queued: notifications.value.filter(item => ['accepted', 'scheduled', 'queued', 'processing', 'retrying'].includes(item.status)).length,
  sent: notifications.value.filter(item => item.status === 'sent').length,
  exceptions: notifications.value.filter(item => ['failed', 'suppressed'].includes(item.status)).length,
}))

async function refreshData() {
  loading.value = true
  errorMessage.value = ''
  try {
    const [recipientResult, templateResult] = await Promise.all([api.recipients(), api.templates()])
    recipients.value = recipientResult.items
    templates.value = templateResult.items
    if (!selectedRecipient.value && recipients.value.length) selectedRecipient.value = recipients.value[0].id
    if (!selectedTemplate.value && channelTemplates.value.length) selectedTemplate.value = channelTemplates.value[0].template_id
    await Promise.all([refreshNotifications(), refreshPreferences()])
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Could not load the operator workspace.'
  } finally {
    loading.value = false
  }
}

async function refreshNotifications() {
  if (!selectedRecipient.value) return
  try { notifications.value = (await api.notifications(selectedRecipient.value)).items }
  catch (error) { errorMessage.value = error instanceof Error ? error.message : 'Could not load notifications.' }
}

async function refreshPreferences() {
  if (!selectedRecipient.value) return
  try {
    const values = await Promise.all(channels.map(channel => api.preference(selectedRecipient.value, channel)))
    for (const preference of values) preferences.value[preference.channel] = preference.opt_in
  } catch (error) { errorMessage.value = error instanceof Error ? error.message : 'Could not load preferences.' }
}

watch(selectedRecipient, async () => { await Promise.all([refreshNotifications(), refreshPreferences()]) })
watch(selectedChannel, () => {
  selectedTemplate.value = channelTemplates.value[0]?.template_id || ''
})
watch(selectedTemplate, id => {
  const template = channelTemplates.value.find(item => item.template_id === id)
  if (template) { subject.value = template.subject; body.value = template.body }
})

async function togglePreference(channel: Channel, checked: boolean) {
  preferenceSaving.value = true
  errorMessage.value = ''
  try {
    const preference = await api.setPreference(selectedRecipient.value, channel, checked)
    preferences.value[preference.channel] = preference.opt_in
    notice.value = `${channelLabels[channel]} preference ${checked ? 'enabled' : 'paused'}.`
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Preference could not be saved.'
  } finally { preferenceSaving.value = false }
}

function useTemplate() {
  const template = channelTemplates.value.find(item => item.template_id === selectedTemplate.value)
  if (template) { subject.value = template.subject; body.value = template.body }
}

function focusRecipient() {
  document.getElementById('recipient-select')?.focus()
}

async function submitNotification() {
  errorMessage.value = ''
  notice.value = ''
  if (!selectedRecipient.value) { errorMessage.value = 'Select a recipient first.'; return }
  if (!body.value.trim()) { errorMessage.value = 'Message body is required.'; return }
  let variables: Record<string, unknown>
  try {
    variables = JSON.parse(variablesText.value || '{}') as Record<string, unknown>
    if (Array.isArray(variables) || variables === null || typeof variables !== 'object') throw new Error('Variables must be a JSON object.')
  } catch { errorMessage.value = 'Variables must be a valid JSON object.'; return }
  if (scheduleEnabled.value && !scheduledAt.value) { errorMessage.value = 'Choose a delivery time for this scheduled notification.'; return }

  submitting.value = true
  try {
    const template = channelTemplates.value.find(item => item.template_id === selectedTemplate.value && item.body === body.value && item.subject === subject.value)
    const payload: Record<string, unknown> = {
      recipient_id: selectedRecipient.value,
      channel: selectedChannel.value,
      variables,
      simulation: simulation.value,
      ...(scheduleEnabled.value ? { scheduled_at: new Date(scheduledAt.value).toISOString() } : {}),
      ...(template ? { template_id: template.template_id, template_version: template.version } : { subject: subject.value, body: body.value }),
    }
    const accepted = await api.createNotification(payload, crypto.randomUUID())
    notice.value = `Request ${accepted.notification_id.slice(0, 8)} accepted as ${accepted.status}.`
    await refreshNotifications()
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : 'Notification could not be submitted.'
  } finally { submitting.value = false }
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value))
}
function channelIcon(channel: Channel) {
  return ({ ios_push: '◉', android_push: '◈', sms: '↗', email: '✉' })[channel]
}
function statusClass(status: string) { return `status status-${status}` }

onMounted(refreshData)
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar" aria-label="Main navigation">
      <a class="brand" href="#overview" @click.prevent="activeView = 'overview'">
        <span class="brand-mark" aria-hidden="true">N</span>
        <span><strong>northstar</strong><small>notification ops</small></span>
      </a>
      <div class="workspace-label">WORKSPACE</div>
      <nav>
        <button class="nav-item" :class="{ active: activeView === 'overview' }" @click="activeView = 'overview'"><span>▦</span> Overview</button>
        <button class="nav-item" :class="{ active: activeView === 'compose' }" @click="activeView = 'compose'"><span>＋</span> Compose</button>
        <button class="nav-item" :class="{ active: activeView === 'recipients' }" @click="activeView = 'recipients'"><span>♙</span> Recipients</button>
        <button class="nav-item" :class="{ active: activeView === 'templates' }" @click="activeView = 'templates'"><span>▤</span> Templates <span class="nav-count">{{ templates.length }}</span></button>
      </nav>
      <div class="sidebar-bottom">
        <div class="environment-card"><span class="live-dot"></span><div><strong>Local sandbox</strong><small>Synthetic data only</small></div></div>
        <a class="docs-link" href="https://bytebytego.com/courses/system-design-interview/design-a-notification-system" target="_blank" rel="noreferrer">System design source <span>↗</span></a>
        <div class="profile"><div class="avatar">MM</div><div><strong>Marcelo Miyake</strong><small>Operator</small></div><span class="profile-menu">···</span></div>
      </div>
    </aside>

    <main class="main-area">
      <header class="topbar">
        <div class="crumb"><span>Workspace</span><b>/</b><strong>{{ activeView === 'overview' ? 'Overview' : activeView[0]?.toUpperCase() + activeView.slice(1) }}</strong></div>
        <div class="top-actions"><span class="secure-label"><span>●</span> Private environment</span><button class="icon-button" aria-label="Refresh data" @click="refreshData">↻</button></div>
      </header>

      <div class="content-wrap">
        <div v-if="errorMessage" class="alert alert-error" role="alert"><span aria-hidden="true">!</span>{{ errorMessage }}<button aria-label="Dismiss error" @click="errorMessage = ''">×</button></div>
        <output v-if="notice" class="alert alert-success"><span aria-hidden="true">✓</span>{{ notice }}<button aria-label="Dismiss message" @click="notice = ''">×</button></output>

        <section class="page-intro">
          <div><div class="eyebrow">OPERATIONS / 24 SEP 2026</div><h1>Good morning, Marcelo <span>✳</span></h1><p>Here's the latest on your notification delivery pipeline.</p></div>
          <button class="button-primary" @click="activeView = 'compose'; focusRecipient()"><span>＋</span> Create notification</button>
        </section>

        <section class="metric-grid" aria-label="Delivery metrics">
          <article class="metric-card"><div class="metric-head"><span>Requests today</span><span class="metric-icon violet">↗</span></div><div class="metric-value">{{ counts.total.toLocaleString() }}<small> requests</small></div><div class="metric-foot"><span class="metric-neutral">Local sample</span><span>Current workspace</span></div><div class="sparkline spark-violet" aria-hidden="true"><i v-for="n in 16" :key="n" :style="{ height: `${12 + ((n * 17) % 38)}px` }"></i></div></article>
          <article class="metric-card"><div class="metric-head"><span>In the queue</span><span class="metric-icon amber">◷</span></div><div class="metric-value">{{ counts.queued.toLocaleString() }}<small> active</small></div><div class="metric-foot"><span class="metric-neutral">Per-channel queues</span><span>Outbox backed</span></div><div class="sparkline spark-amber" aria-hidden="true"><i v-for="n in 16" :key="n" :style="{ height: `${8 + ((n * 11) % 37)}px` }"></i></div></article>
          <article class="metric-card"><div class="metric-head"><span>Provider accepted</span><span class="metric-icon green">✓</span></div><div class="metric-value">{{ counts.sent.toLocaleString() }}<small> recorded</small></div><div class="metric-foot"><span class="metric-positive">Simulated acceptance</span><span>Not recipient delivery</span></div><div class="sparkline spark-green" aria-hidden="true"><i v-for="n in 16" :key="n" :style="{ height: `${13 + ((n * 13) % 38)}px` }"></i></div></article>
          <article class="metric-card"><div class="metric-head"><span>Needs attention</span><span class="metric-icon red">!</span></div><div class="metric-value">{{ counts.exceptions.toLocaleString() }}<small> exceptions</small></div><div class="metric-foot"><span class="metric-neutral">Suppressed or failed</span><span>Retry policy enabled</span></div><div class="sparkline spark-red" aria-hidden="true"><i v-for="n in 16" :key="n" :style="{ height: `${6 + ((n * 7) % 28)}px` }"></i></div></article>
        </section>

        <div class="section-row"><div><h2>Send a notification</h2><p>Submit a synthetic request to the local delivery pipeline.</p></div><span class="step-chip"><span class="step-dot"></span> Ready to send</span></div>
        <div class="work-grid">
          <section class="panel compose-panel" aria-labelledby="compose-title">
            <div class="panel-header"><div class="panel-icon">✉</div><div><h3 id="compose-title">New message</h3><p>Client-triggered or scheduled delivery</p></div><span class="panel-menu">···</span></div>
            <form @submit.prevent="submitNotification">
              <div class="form-row two-col">
                <label class="field">Recipient <select id="recipient-select" v-model="selectedRecipient" :disabled="loading"><option value="" disabled>Select recipient</option><option v-for="recipient in recipients" :key="recipient.id" :value="recipient.id">{{ recipient.display_name }}</option></select><small v-if="recipients[0]">{{ recipients.find(item => item.id === selectedRecipient)?.email_masked || 'Synthetic recipient' }}</small></label>
                <label class="field">Delivery channel <select v-model="selectedChannel"><option v-for="channel in channels" :key="channel" :value="channel">{{ channelIcon(channel) }} &nbsp;{{ channelLabels[channel] }}</option></select><small>Each channel has its own queue</small></label>
              </div>
              <div class="field template-field"><label for="template-select">Template <span class="optional">OPTIONAL</span></label><div class="inline-field"><select id="template-select" v-model="selectedTemplate"><option value="">Write a custom message</option><option v-for="template in channelTemplates" :key="`${template.template_id}-${template.version}`" :value="template.template_id">{{ template.name }} · v{{ template.version }}</option></select><button type="button" class="button-quiet" @click="useTemplate">Apply</button></div></div>
              <div v-if="selectedChannel === 'email'" class="field"><label for="subject-input">Subject</label><input id="subject-input" v-model="subject" maxlength="200" placeholder="A clear, helpful subject line" /></div>
              <div class="field"><label for="body-input">Message body</label><textarea id="body-input" v-model="body" rows="4" maxlength="10000" placeholder="Write a concise message. Use {{'{{variable}}'}} for template values."></textarea><small class="field-hint">{{ body.length }} / 10,000 characters · Variables are rendered by the worker.</small></div>
              <div class="field"><label for="variables-input">Template variables <span class="optional">JSON</span></label><textarea id="variables-input" v-model="variablesText" class="variables-input" rows="3" spellcheck="false" aria-describedby="variables-help"></textarea><small id="variables-help" class="field-hint">Synthetic test data only. Example: { "order_id": "A-1042" }</small></div>
              <details class="simulation-details"><summary>Local adapter scenario <span>TEST ONLY</span></summary><label class="field" for="simulation-select">Simulated provider response<select id="simulation-select" v-model="simulation"><option value="success">Accept request</option><option value="transient_failure">Transient failure, then retry successfully</option><option value="permanent_failure">Permanent failure, send to dead letter</option></select><small>Recording adapter control; no real provider is called.</small></label></details>
              <div class="schedule-row"><label class="switch-label"><input v-model="scheduleEnabled" type="checkbox" /><span class="switch-track"></span><span><strong>Schedule for later</strong><small>Delivered when the scheduled time arrives</small></span></label><label v-if="scheduleEnabled" class="schedule-date" for="scheduled-at">Send at<input id="scheduled-at" v-model="scheduledAt" type="datetime-local" /></label></div>
              <div class="form-footer"><span class="private-note"><span>♢</span> Request is private to this local sandbox</span><button class="button-primary send-button" type="submit" :disabled="submitting || loading"><span>{{ submitting ? '◌' : '↗' }}</span>{{ submitting ? 'Submitting…' : scheduleEnabled ? 'Schedule notification' : 'Send notification' }}</button></div>
            </form>
          </section>

          <aside class="side-stack" aria-label="Notification controls">
            <section class="panel preference-panel" aria-labelledby="preferences-title">
              <div class="panel-header"><div class="panel-icon preference-icon">⚙</div><div><h3 id="preferences-title">Recipient preferences</h3><p>{{ selectedName }} · opt-in by channel</p></div><button class="more-button" aria-label="Refresh preferences" @click="refreshPreferences">↻</button></div>
              <div class="preference-list"><label v-for="channel in channels" :key="channel" class="preference-item"><span class="channel-icon" :class="`channel-${channel}`">{{ channelIcon(channel) }}</span><span class="preference-name"><strong>{{ channelLabels[channel] }}</strong><small>{{ channel === 'ios_push' || channel === 'android_push' ? 'Device notifications' : `${channelLabels[channel]} delivery` }}</small></span><span class="preference-state">{{ preferences[channel] ? 'On' : 'Paused' }}</span><input class="preference-switch" :aria-label="`${channelLabels[channel]} opt in`" type="checkbox" :checked="preferences[channel]" :disabled="preferenceSaving || !selectedRecipient" @change="togglePreference(channel, ($event.target as HTMLInputElement).checked)" /></label></div>
              <div class="preference-note"><span>ⓘ</span><p>Opt-outs are checked now and again just before the worker records provider acceptance.</p></div>
            </section>

            <section class="panel activity-panel" aria-labelledby="activity-title">
              <div class="panel-header"><div><h3 id="activity-title">Recent activity</h3><p>Latest requests for {{ selectedName }}</p></div><button class="text-button" @click="activeView = 'history'">View all <span>→</span></button></div>
              <div v-if="loading" class="empty-state"><span class="loader"></span><p>Loading activity…</p></div>
              <div v-else-if="!recentNotifications.length" class="empty-state"><span class="empty-icon">↗</span><p>No notifications yet</p><small>Your recent requests will appear here.</small></div>
              <ol v-else class="activity-list"><li v-for="item in recentNotifications" :key="item.id" class="activity-item"><span class="activity-channel" :class="`channel-${item.channel}`">{{ channelIcon(item.channel) }}</span><span class="activity-content"><strong>{{ channelLabels[item.channel] }} notification</strong><small>{{ formatDate(item.created_at) }}</small></span><span :class="statusClass(item.status)">{{ item.status }}</span></li></ol>
              <div class="activity-footer"><span><span class="live-dot"></span> Status refreshes on request</span><button aria-label="Refresh activity" @click="refreshNotifications">↻</button></div>
            </section>

            <section class="scenario-note"><div class="scenario-glyph">↗</div><div><strong>Design scenario</strong><p>16M notifications/day is the course planning scenario. This local cluster does not benchmark or prove that capacity.</p><a href="https://bytebytego.com/courses/system-design-interview/design-a-notification-system" target="_blank" rel="noreferrer">Read source chapter <span>↗</span></a></div></section>
          </aside>
        </div>
        <footer class="page-footer"><span>Notification System · local reference implementation</span><span>Provider acceptance is simulated · <a href="../docs/system-design.md">System design</a></span></footer>
      </div>
    </main>
  </div>
</template>
