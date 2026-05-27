import type { SmallTalkNotesDecryptedNote } from '$lib/types/smalltalk.types.js'
import { auth, refresh, authApi, global_private_key } from './auth.svelte'
import { addNotification } from './notifications.svelte.js'
import { getDb } from './sqlite.js'

// --- state ---

export const db_state = $state({
  ready: false,
  needs_onboarding: false,   // no public key on account yet
  needs_key_sync: false,     // has public key but no local private key
})

let db: any = null  // sqlite db instance

// --- migrations ---

const MIGRATIONS: string[][] = [
  [`
  CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
  )`,
    `
  CREATE TABLE IF NOT EXISTS contacts (
    user_id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    public_key TEXT,
    last_seen TEXT
  )`,],

  /*  [`
    CREATE TABLE IF NOT EXISTS smalltalk_decrypted_notes (
          id TEXT PRIMARY KEY,
          user_id TEXT NOT NULL,
          group_id TEXT,
          dec_name TEXT NOT NULL,
          dec_content TEXT,
          is_protected INTEGER NOT NULL DEFAULT 0,
          password_hash TEXT,
          salt TEXT,
          rank INTEGER NOT NULL DEFAULT 0,
          is_deleted INTEGER NOT NULL DEFAULT 0,
          created_at INTEGER NOT NULL,
          updated_at INTEGER NOT NULL,
          last_accessed_at INTEGER NOT NULL
      )`,], */

  // maybe add "is_online INTEGER DEFAULT 0" to contacts?

  /* `CREATE TABLE IF NOT EXISTS conversations (
     id TEXT PRIMARY KEY,
     partner_id TEXT NOT NULL,
     partner_username TEXT NOT NULL,
     partner_public_key TEXT,
     created_at TEXT NOT NULL
   );
 
   CREATE TABLE IF NOT EXISTS group_chats (
     id TEXT PRIMARY KEY,
     name TEXT NOT NULL,
     owner_id TEXT NOT NULL,
     created_at TEXT NOT NULL
   );
 
   CREATE TABLE IF NOT EXISTS group_keys (
     gc_id TEXT NOT NULL,
     group_key TEXT NOT NULL,
     version INTEGER NOT NULL,
     PRIMARY KEY (gc_id, version)
   );
 
   CREATE TABLE IF NOT EXISTS messages (
     id TEXT PRIMARY KEY,
     conversation_id TEXT,
     gc_id TEXT,
     sender_id TEXT NOT NULL,
     sender_username TEXT NOT NULL,
     plaintext TEXT NOT NULL,
     created_at TEXT NOT NULL,
     edited_at TEXT,
     deleted INTEGER DEFAULT 0,
     status TEXT DEFAULT 'delivered'
   );`, */
  [
    `INSERT OR IGNORE INTO meta (key, value) VALUES ('notes_last_synced_at', '1970-01-01T00:00:00Z');`,
    `INSERT OR IGNORE INTO meta (key, value) VALUES ('messages_last_synced_at', '1970-01-01T00:00:00Z');`
  ],
]

async function run_migrations() {
  let current_version = 0
  try {
    const rows = await db.exec(
      `SELECT value FROM meta WHERE key = 'db_version'`
    )
    if (rows.length > 0) {
      current_version = parseInt(rows[0].value)
    }
  } catch {
    current_version = 0
  }

  for (let i = current_version; i < MIGRATIONS.length; i++) {
    for (const sql of MIGRATIONS[i]) {
      await db.exec(sql)
    }
    await db.exec(
      `INSERT OR REPLACE INTO meta(key, value) VALUES('db_version', ?)`,
      [String(i + 1)]
    )
    console.log(`migration ${i + 1} done`)
  }
}

// --- private key management ---

async function check_private_key(): Promise<'ready' | 'needs_onboarding' | 'needs_sync'> {
  const has_server_key = !!auth.public_key

  if (!has_server_key) {
    return 'needs_onboarding'
  }

  // does the private key exist locally?
  const local_key = await get_private_key_from_indexeddb()
  if (!local_key) {
    // has account key but not on this device
    return 'needs_sync'
  }

  return 'ready'
}

export async function get_private_key_from_indexeddb(): Promise<CryptoKey | null> {
  return new Promise((resolve) => {
    const request = indexedDB.open('smalltalk_keys', 1)

    request.onupgradeneeded = (e) => {
      const db = (e.target as IDBOpenDBRequest).result
      if (!db.objectStoreNames.contains('keys')) {
        db.createObjectStore('keys')
      }
    }

    request.onsuccess = async (e) => {
      const idb = (e.target as IDBOpenDBRequest).result
      const tx = idb.transaction('keys', 'readonly')
      const store = tx.objectStore('keys')
      const get = store.get(`private_key_${auth.id} `)

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

export async function store_private_key_in_indexeddb(key: CryptoKey): Promise<void> {
  const exported = await crypto.subtle.exportKey('pkcs8', key)

  return new Promise((resolve, reject) => {
    const request = indexedDB.open('smalltalk_keys', 1)

    request.onupgradeneeded = (e) => {
      const db = (e.target as IDBOpenDBRequest).result
      if (!db.objectStoreNames.contains('keys')) {
        db.createObjectStore('keys')
      }
    }

    request.onsuccess = (e) => {
      const idb = (e.target as IDBOpenDBRequest).result
      const tx = idb.transaction('keys', 'readwrite')
      const store = tx.objectStore('keys')
      store.put(exported, `private_key_${auth.id} `)
      tx.oncomplete = () => resolve()
      tx.onerror = () => reject(tx.error)
    }

    request.onerror = () => reject(request.error)
  })
}

export async function generate_and_store_keypair(): Promise<{ public_key: string }> {
  const keypair = await crypto.subtle.generateKey(
    { name: 'ECDH', namedCurve: 'P-256' },
    true,
    ['deriveKey', 'deriveBits']
  )

  await store_private_key_in_indexeddb(keypair.privateKey)

  const public_key_buffer = await crypto.subtle.exportKey('spki', keypair.publicKey)
  const public_key_b64 = btoa(String.fromCharCode(...new Uint8Array(public_key_buffer)))

  let patchPubkey = authApi.path(`/me`).method("patch").create();
  await patchPubkey({ "public_key": public_key_b64 });

  await refresh();

  addNotification({
    type: "important_info",
    title: "Notification",
    body: "Encryption Key added! Refresh the page for changes to take place.",
    sender: "DevinLittle.Net",
    global: false,
  });

  global_private_key.value = keypair.privateKey;

  return { public_key: public_key_b64 }
}

// --- main mountDB function ---

export async function mountDB() {
  db = await getDb();
  await run_migrations()

  const key_status = await check_private_key()

  if (key_status === 'needs_onboarding') {
    db_state.needs_onboarding = true
    db_state.ready = false
    // ui will show onboarding modal
    // contacts still populate; messaging disabled
    return
  }

  if (key_status === 'needs_sync') {
    db_state.needs_key_sync = true
    db_state.ready = false
    // ui will show "sync from another device" modal
    // contacts still populate, but messaging disabled
    return
  }

  // all good!
  db_state.ready = true
}

// --- sqlite query helpers ---

export async function db_exec(sql: string, params: any[] = []) {
  if (!db) throw new Error('db not mounted')
  return db.exec(sql, params)
}

export async function db_run(sql: string, params: any[] = []) {
  if (!db) throw new Error('db not mounted')
  return db.run(sql, params)
}

// --- contact helpers ---

export function upsert_contact(contact: {
  user_id: string
  username: string
  public_key?: string | null
  last_seen?: string | null
}) {
  db_run(
    `INSERT INTO contacts(user_id, username, public_key, last_seen)
VALUES(?, ?, ?, ?)
     ON CONFLICT(user_id) DO UPDATE SET
username = excluded.username,
  public_key = COALESCE(excluded.public_key, contacts.public_key),
  last_seen = COALESCE(excluded.last_seen, contacts.last_seen); `,
    [
      contact.user_id,
      contact.username,
      contact.public_key ?? null,
      contact.last_seen ?? null,
    ]
  )
}

// TODO: implement the set online in notifications and the frontent websocket router
export function set_contact_online(user_id: string, is_online: boolean) {
  db_run(
    `UPDATE contacts SET is_online = ? WHERE user_id = ? `,
    [is_online ? 1 : 0, user_id]
  )
}

// TODO: implement the set online in notifications and the frontent websocket router
export function set_contact_last_seen(user_id: string, last_seen: string) {
  db_run(
    `UPDATE contacts SET last_seen = ?, is_online = 0 WHERE user_id = ? `,
    [last_seen, user_id]
  )
}

// --- message helpers ---
/* 
export function upsert_message(msg: {
  id: string
  conversation_id: string | null
  gc_id: string | null
  sender_id: string
  sender_username: string
  plaintext: string
  created_at: string
  edited_at?: string | null
  deleted?: boolean
  status?: string
}) {
  db_run(
    `INSERT INTO messages(id, conversation_id, gc_id, sender_id, sender_username, plaintext, created_at, edited_at, deleted, status)
VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
     ON CONFLICT(id) DO UPDATE SET
plaintext = excluded.plaintext,
  edited_at = excluded.edited_at,
  deleted = excluded.deleted,
  status = excluded.status`,
    [
      msg.id,
      msg.conversation_id ?? null,
      msg.gc_id ?? null,
      msg.sender_id,
      msg.sender_username,
      msg.plaintext,
      msg.created_at,
      msg.edited_at ?? null,
      msg.deleted ? 1 : 0,
      msg.status ?? 'delivered'
    ]
  )
}

export function get_messages(conversation_id: string): any[] {
  return db_exec(
    `SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at ASC`,
    [conversation_id]
  )
}

export function get_gc_messages(gc_id: string): any[] {
  return db_exec(
    `SELECT * FROM messages WHERE gc_id = ? ORDER BY created_at ASC`,
    [gc_id]
  )
}

// --- sync helpers ---

export function get_last_synced_at(): string {
  const rows = db_exec(`SELECT value FROM meta WHERE key = 'last_synced_at'`)
  return rows[0]?.value ?? '1970-01-01T00:00:00Z'
}

export function set_last_synced_at(timestamp: string) {
  db_run(
    `INSERT OR REPLACE INTO meta(key, value) VALUES('messages_last_synced_at', ?)`,
    [timestamp]
  )
}

// --- group key helpers ---

export function store_group_key(gc_id: string, group_key: string, version: number) {
  db_run(
    `INSERT OR IGNORE INTO group_keys(gc_id, group_key, version) VALUES(?, ?, ?)`,
    [gc_id, group_key, version]
  )
}

export function get_group_key(gc_id: string, version: number): string | null {
  const rows = db_exec(
    `SELECT group_key FROM group_keys WHERE gc_id = ? AND version = ? `,
    [gc_id, version]
  )
  return rows[0]?.group_key ?? null
}

export function get_latest_group_key(gc_id: string): { group_key: string, version: number } | null {
  const rows = db_exec(
    `SELECT group_key, version FROM group_keys WHERE gc_id = ? ORDER BY version DESC LIMIT 1`,
    [gc_id]
  )
  return rows[0] ?? null
}

*/
