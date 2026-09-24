export const channels = ['ios_push', 'android_push', 'sms', 'email'] as const
export type Channel = (typeof channels)[number]

export interface Recipient {
  id: string
  display_name: string
  email_masked?: string
  phone_masked?: string
}

export interface Template {
  template_id: string
  version: number
  name: string
  channel: Channel
  subject: string
  body: string
}

export interface Notification {
  id: string
  recipient_id: string
  channel: Channel
  status: string
  attempts: number
  scheduled_at?: string | null
  created_at: string
  updated_at: string
  last_error?: string | null
}

export interface Preference {
  recipient_id: string
  channel: Channel
  opt_in: boolean
}
