import { sendMessage } from "./notifications.svelte"
import { auth, API_URL, authFetch } from "./auth.svelte.js"
import { store_private_key_in_indexeddb } from "./sqlite.svelte"
import type { KeySyncMessage, KeySyncPayload, KeySyncStatus, PendingChallenge, } from "$lib/utils/smalltalk.types.js"
import wordlist from "$lib/utils/wordlist.json"

// --- emoji pool ---

// just wanna say thank you gemini for giving these emojis!
const EMOJI_POOL = [
  // Nature & Animals
  '🔥', '🌊', '⚡', '🌙', '🍄', '🦋', '🦊', '🍀', '🌺', '🦁', '🌸', '🦄',
  '☀️', '🪐', '🌟', '🌪️', '🌈', '🐾', '🌲', '🌻', '🍁', '🐚', '🦜', '🐘',
  '🐙', '🐳', '🐝', '🦉', '🦖', '🐉', '🌵', '🌴', '🐧', '🐢', '🦢', '🦌',
  '🐒', '🦓', '🦒', '🦩', '🦚', '🦦', '🦥', '🐕', '🐈', '🐇', '🐹', '🐼',
  '🐨', '🐸', '🦂', '🕸️', '🌍', '🌋', '🗻', '🏜️', '🏝️', '☄️', '🌌', '❄️',

  // Food & Drink
  '🍕', '🍦', '🍩', '🍣', '🌮', '🍔', '🥑', '🍓', '🍒', '🍍', '🍉', '🥨',
  '🥐', '🥞', '🍜', '🍱', '🍿', '🍰', '🍪', '🍫', '🍯', '🍷', '🍺', '🍹',
  '🥤', '🍵', '🥡', '🍳', '🧀', '🍄', '🌶️', '🌽', '🍎', '🍇', '🍋', '🍐',
  '🍑', '🥭', '🥥', '🥦', '🥯', '🥖', '🍖', '🍤', '🍮', '🍭', '🍧', '🍢',

  // Activities, Sports & Hobbies
  '🎯', '🎸', '🎲', '🎪', '🎭', '🎨', '🚀', '🎵', '🎮', '🧩', '🎬', '📸',
  '🛹', '🚲', '⚽', '🏀', '🎾', '🥊', '🏹', '🎣', '⛷️', '🏄', '🧗', '🧘',
  '🏋️', '🏇', '🛶', '🏎️', '⛳', '🎳', '🎷', '🎻', '🎹', '🥁', '🎟️', '🎡',
  '🎢', '🎠', '🛸', '🛰️', '🛶', '⛵', '🗺️', '🏰', '🏮', '🎐', '🧧', '💎',

  // Objects & Symbols
  '🔮', '🧿', '🧬', '🔭', '🧪', '🔑', '🗝️', '📜', '🕯️', '💡', '⏰', '🔋',
  '📱', '💻', '🖨️', '💾', '📼', '📷', '📽️', '📖', '📂', '📦', '📫', '🎁',
  '🎈', '🎉', '🎊', '🎏', '🎐', '🎀', '🪄', '🧿', '🔮', '🧸', '🪩', '🪅',
  '🧹', '🧼', '🧪', '🔭', '🧲', '⚖️', '⛓️', '🛠️', '⚙️', '🧱', '💎', '💰',

  // People & Expressions
  '✨', '💎', '👑', '👒', '🕶️', '🧣', '👟', '🎒', '💍', '💄', '🧶', '🧵',
  '🫀', '🧠', '👀', '👅', '🤘', '🤙', '🤟', '👏', '🤝', '🙌', '🧚', '🧜',
  '🧛', '🧟', '🧞', '👻', '👽', '🤖', '👾', '🤡', '👹', '👺', '🎭', '🧶',

  // Shapes & Abstract
  '❤️', '🧡', '💛', '💚', '💙', '💜', '🖤', '🤍', '🤎', '💖', '💝', '💢',
  '💥', '💫', '💨', '💦', '💬', '🗯️', '💭', '💤', '💮', '💯', '🉐', '㊙️',
  '🌀', '🛑', '⚠️', '🔱', '⚜️', '🔰', '♻️', '✅', '💹', '🌐', '💠', '⚛️',
  '🕉️', '✡️', '☸️', '☯️', '✝️', '☦️', '☪️', '🕎', '🔯', '♈', '♉', '♊',
  '♋', '♌', '♍', '♎', '♏', '♐', '♑', '♒', '♓', '⛎', '🔀', '🔁', '🔃'
];

// --- state ---
export const keysync = $state({
  status: 'idle' as KeySyncStatus,
  challenge_emojis: [] as string[],
  correct_emoji: null as string | null, // only known by TD, never sent
  pending_challenge: null as PendingChallenge | null,
})


export function handleKeySync(msg: KeySyncMessage) {
  // drop if WE sent this AND it was targeted
  if (msg.from_session_id === auth.session_id) return

  // drop targeted messages not meant for this device
  if (msg.target_session_id !== null && msg.target_session_id !== auth.session_id) return

  const payload = msg.payload;
  switch (payload.type) {
    case 'KeySyncRequest': return handleKeySyncRequest(msg.from_session_id)
    case 'KeySyncChallenge': return handleKeySyncChallenge(payload.emojis)
    case 'KeySyncResponse': return handleKeySyncResponse(payload.chosen, msg.from_session_id)
    case 'KeySyncHandover': return handleKeySyncHandover(payload.private_key)
    case 'KeySyncFailed': return handleKeySyncFailed()
    case 'KeySyncComplete': return handleKeySyncComplete()
    case 'KeySyncExpired': return handleKeySyncExpired()
  }
}

function pick_random_emojis(count: number): string[] {
  const shuffled = [...EMOJI_POOL].sort(() => Math.random() - 0.5)
  return shuffled.slice(0, count)
}

function arraybuffer_to_base64(buffer: ArrayBuffer): string {
  return btoa(String.fromCharCode(...new Uint8Array(buffer)))
}

function base64_to_arraybuffer(b64: string): ArrayBuffer {
  const binary = atob(b64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes.buffer
}

// --- new device side ---

// ND: broadcast to all user devices that we need a key
export function requestKeySync() {
  keysync.status = 'requesting'
  sendKeySync({ type: 'KeySyncRequest' })
}

// ND: received emojis from TD, show them to user
function handleKeySyncChallenge(emojis: string[]) {
  keysync.challenge_emojis = emojis
  keysync.status = 'pending_click'
}

// ND: user clicked an emoji, send response to TD
export function respondToChallenge(emoji: string) {
  if (keysync.status !== 'pending_click') return
  sendKeySync({
    type: 'KeySyncResponse',
    chosen: emoji
  }, keysync.pending_challenge?.requester_session_id ?? null)
}

// ND: received the private key from TD
async function handleKeySyncHandover(private_key_b64: string) {
  keysync.status = 'transferring'
  try {
    const key_buffer = base64_to_arraybuffer(private_key_b64)

    const private_key = await crypto.subtle.importKey(
      'pkcs8',
      key_buffer,
      { name: 'ECDH', namedCurve: 'P-256' },
      true,
      ['deriveKey', 'deriveBits']
    )

    await store_private_key_in_indexeddb(private_key)

    sendKeySync({ type: 'KeySyncComplete' })
    keysync.status = 'complete'
  } catch (e) {
    console.error('failed to store key:', e)
    keysync.status = 'failed'
  }
}

// ND: wrong emoji, try again
function handleKeySyncFailed() {
  keysync.status = 'failed'
  keysync.challenge_emojis = []
  // UI can show "incorrect, try again" and call requestKeySync() again
}

// ND: challenge timed out
function handleKeySyncExpired() {
  keysync.status = 'expired'
  keysync.challenge_emojis = []
}

// --- trusted device side ---

// TD: received a sync request from another device
function handleKeySyncRequest(requester_session_id: string) {
  const emojis = pick_random_emojis(5)
  const correct_emoji = emojis[Math.floor(Math.random() * emojis.length)]

  const timeout_id = setTimeout(() => {
    if (keysync.pending_challenge) {
      sendKeySync({ type: 'KeySyncExpired' }, requester_session_id)
      keysync.pending_challenge = null
      keysync.status = 'idle'
      keysync.correct_emoji = null
    }
  }, 60000) // 60 second expiry

  keysync.pending_challenge = {
    correct_emoji,
    emojis,
    requester_session_id,
    expires_at: Date.now() + 60000,
    timeout_id
  }

  keysync.correct_emoji = correct_emoji  // shown highlighted on TD screen
  keysync.challenge_emojis = emojis      // shown on TD screen too
  keysync.status = 'challenging'

  // send emojis to ND (correct one NOT included)
  sendKeySync({
    type: 'KeySyncChallenge',
    emojis
  }, requester_session_id)
}

// TD: ND clicked an emoji, validate it
async function handleKeySyncResponse(chosen: string, requester_session_id: string) {
  const challenge = keysync.pending_challenge
  if (!challenge) return

  // clear the timeout
  clearTimeout(challenge.timeout_id)
  keysync.pending_challenge = null
  keysync.correct_emoji = null

  if (chosen !== challenge.correct_emoji) {
    // wrong emoji
    sendKeySync({ type: 'KeySyncFailed' }, requester_session_id)
    keysync.status = 'idle'
    keysync.challenge_emojis = []
    return
  }

  // correct emoji clicked! export and send the private key
  keysync.status = 'transferring'

  try {
    const private_key = await get_private_key_from_indexeddb_raw()
    if (!private_key) {
      sendKeySync({ type: 'KeySyncFailed' }, requester_session_id)
      keysync.status = 'failed'
      return
    }

    // export to ArrayBuffer → base64
    const exported = await crypto.subtle.exportKey('pkcs8', private_key)
    const b64 = arraybuffer_to_base64(exported)

    sendKeySync({
      type: 'KeySyncHandover',
      private_key: b64
    }, requester_session_id)

  } catch (e) {
    console.error('failed to export key:', e)
    sendKeySync({ type: 'KeySyncFailed' }, requester_session_id)
    keysync.status = 'failed'
  }
}

// TD: ND confirmed they got the key
function handleKeySyncComplete() {
  keysync.status = 'complete'
  keysync.challenge_emojis = []
  // UI shows "key transferred successfully"
  // reset after a few seconds
  setTimeout(() => {
    keysync.status = 'idle'
  }, 3000)
}

// --- recovery phrase ---

// generate 12 random BIP39-style words
export function generate_recovery_words(): string[] {
  const words: string[] = []
  for (let i = 0; i < 12; i++) {
    const idx = crypto.getRandomValues(new Uint32Array(1))[0] % wordlist.length
    words.push(wordlist[idx])
  }
  return words
}

// derive an AES key from the 12 words + user_id as salt
async function derive_key_from_words(words: string[]): Promise<CryptoKey> {
  const phrase = words.join(',')
  const key_material = await crypto.subtle.importKey(
    'raw',
    new TextEncoder().encode(phrase),
    'PBKDF2',
    false,
    ['deriveKey']
  )
  return crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt: new TextEncoder().encode(auth.id!),
      iterations: 100000,
      hash: 'SHA-256'
    },
    key_material,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt', 'decrypt']
  )
}

// hash the words for server verification
async function hash_words(words: string[]): Promise<string> {
  const phrase = words.join(',') + auth.id
  const buffer = await crypto.subtle.digest(
    'SHA-256',
    new TextEncoder().encode(phrase)
  )
  return arraybuffer_to_base64(buffer)
}

// setup recovery phrase — call after user confirms they wrote words down
export async function setup_recovery_phrase(words: string[]): Promise<void> {
  const private_key = await get_private_key_from_indexeddb_raw()
  if (!private_key) throw new Error('no private key found')

  const exported = await crypto.subtle.exportKey('pkcs8', private_key)

  const aes_key = await derive_key_from_words(words)

  const iv = crypto.getRandomValues(new Uint8Array(12))
  const encrypted = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv },
    aes_key,
    exported
  )

  const combined = new Uint8Array(iv.length + encrypted.byteLength)
  combined.set(iv, 0)
  combined.set(new Uint8Array(encrypted), iv.length)
  const encrypted_b64 = arraybuffer_to_base64(combined.buffer)

  const recovery_hash = await hash_words(words)

  await authFetch(`${API_URL}/auth/me/recovery`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      recovery_hash,
      encrypted_private_key: encrypted_b64
    })
  })
}

// recover private key using 12 words
export async function recover_with_phrase(words: string[]): Promise<boolean> {
  try {
    const recovery_hash = await hash_words(words)

    const res = await authFetch(`${API_URL}/auth/me/recovery/verify`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ recovery_hash })
    })

    if (!res.ok) return false  // wrong words

    const { encrypted_private_key } = await res.json()

    const combined = base64_to_arraybuffer(encrypted_private_key)

    const iv = combined.slice(0, 12)
    const encrypted_data = combined.slice(12)

    const aes_key = await derive_key_from_words(words)

    const decrypted = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv: new Uint8Array(iv) },
      aes_key,
      encrypted_data
    )

    const private_key = await crypto.subtle.importKey(
      'pkcs8',
      decrypted,
      { name: 'ECDH', namedCurve: 'P-256' },
      true,
      ['deriveKey', 'deriveBits']
    )

    await store_private_key_in_indexeddb(private_key)
    return true

  } catch (e) {
    console.error('recovery failed:', e)
    return false
  }
}

// --- internal helper to get raw CryptoKey from IndexedDB ---
async function get_private_key_from_indexeddb_raw(): Promise<CryptoKey | null> {
  return new Promise((resolve) => {
    const request = indexedDB.open('smalltalk_keys', 1)
    request.onsuccess = async (e) => {
      const idb = (e.target as IDBOpenDBRequest).result
      const tx = idb.transaction('keys', 'readonly')
      const store = tx.objectStore('keys')
      const get = store.get(`private_key_${auth.id}`)
      get.onsuccess = async () => {
        if (!get.result) return resolve(null)
        try {
          const key = await crypto.subtle.importKey(
            'pkcs8',
            get.result,
            { name: 'ECDH', namedCurve: 'P-256' },
            true,
            ['deriveKey', 'deriveBits']
          )
          resolve(key)
        } catch {
          resolve(null)
        }
      }
      get.onerror = () => resolve(null)
    }
    request.onerror = () => resolve(null)
  })
}

export function cancelKeySync() {
  if (keysync.pending_challenge) {
    clearTimeout(keysync.pending_challenge.timeout_id)
    keysync.pending_challenge = null
  }
  keysync.status = 'idle'
  keysync.challenge_emojis = []
  keysync.correct_emoji = null
}

function sendKeySync(payload: KeySyncPayload, target_session_id: string | null = null) {
  sendMessage(JSON.stringify({
    namespace: 'smalltalk_keysync',
    id: crypto.randomUUID(),
    from_session_id: auth.session_id,
    target_session_id,
    payload
  }), auth.id)
}
