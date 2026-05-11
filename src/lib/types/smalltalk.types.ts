
// ND = new device
// TD = trusted device (the device with the private key)

export type KeySyncStatus =
  | 'idle'
  | 'requesting'      // ND sent request, waiting for TD to respond
  | 'challenging'     // TD generated emojis, waiting for ND to click
  | 'pending_click'   // ND showing emojis, waiting for user to click
  | 'transferring'    // TD sending key to ND
  | 'complete'
  | 'failed'
  | 'expired'

export type KeySyncPayload =
  | { type: 'KeySyncRequest'; nd_ephemeral_public_key: string }
  | { type: 'KeySyncChallenge'; emojis: string[] }
  | { type: 'KeySyncResponse'; chosen: string; }
  | { type: 'KeySyncHandover'; private_key: string; td_ephemeral_public_key: string, }
  | { type: 'KeySyncFailed' }
  | { type: 'KeySyncComplete' }
  | { type: 'KeySyncExpired' }

export type KeySyncMessage = {
  namespace: 'keysync'
  from_session_id: string
  target_session_id: string | null
  payload: KeySyncPayload
}

export type PendingChallenge = {
  correct_emoji: string
  emojis: string[]
  requester_session_id: string
  expires_at: number
  timeout_id: ReturnType<typeof setTimeout>
}

export type RecoverySetupResult = {
  words: string[]               // the 12 words to show the user
  recovery_hash: string         // hash to store on server
  encrypted_private_key: string // encrypted blob to store on server
}
