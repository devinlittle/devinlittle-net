import { auth, API_URL, refresh, authFetch } from "./auth.svelte.js";
import { fetchGrades } from "./gradegetter.svelte.js";
import { handleNanoPass } from "./nanopass.svelte.js";
import type { NanoPassMessage, NanoPassPayload } from "./nanopass.types.ts";
import { handleKeySync } from "./smalltalk.svelte.ts";
import type { KeySyncMessage } from "./smalltalk.types.ts";

// --- types ---


type Notification =
  {
    id: string
    type: 'info' | 'global' | "transfer" | "important_info"
    title: string
    body: string
    global?: boolean
    sender?: string | null
    filename?: string
    filesize?: string
    onAccept?: () => void
    onDecline?: () => void
  }

type MessageNamespace = "notification" | "nanopass" | "gradegetter" | "smalltalk_keysync"

type IncomingMessage = {
  namespace: MessageNamespace
  payload: unknown
}

type NotificationPayload = {
  type: "global" | "user"
  title?: string
  content: string
  sender_username?: string
}

// --- state ---

export let notifications = $state<Notification[]>([])
let socket: WebSocket | null = null
let dismissFn: ((id: string) => void) | null = null

// --- notification logic ---

export function registerDismiss(fn: (id: string) => void) {
  dismissFn = fn
}

export function addNotification(notification: Omit<Notification, "id">) {
  const id = crypto.randomUUID()
  const n: Notification = { id, ...notification }
  notifications.push(n)
  if (n.type !== "transfer" && n.type !== "important_info") {
    setTimeout(() => {
      if (dismissFn) dismissFn(id)
    }, 10000)
  }
}
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}



export function removeNotification(id: string) {
  const idx = notifications.findIndex(n => n.id === id)
  if (idx !== -1) notifications.splice(idx, 1)
}

// --- namespace handlers ---

function handleNotification(payload: NotificationPayload) {
  if (payload.type === "global") {
    addNotification({
      type: "global",
      title: payload.title ?? "Global",
      body: payload.content,
      global: true,
    })
    return
  }
  addNotification({
    type: "info",
    title: payload.title ?? "Notification",
    body: payload.content,
    sender: payload.sender_username ?? null,
    global: false,
  })
}

// --- socket ---

function handleMessage(msg: IncomingMessage) {
  switch (msg.namespace) {
    case "notification":
      handleNotification(msg.payload as NotificationPayload)
      break
    case "nanopass":
      handleNanoPass(msg as NanoPassMessage)
      break
    case "gradegetter":
      fetchGrades();
      break
    case "smalltalk_keysync":
      handleKeySync(msg as KeySyncMessage)
      break
  }
}

export function connectNotifications() {
  const isAuthed = auth.id !== null
  const path = isAuthed ? `/ws/${auth.id}` : `/ws/global`
  const url = `${API_URL.replace("https://", "wss://").replace("http://", "ws://")}/notification${path}`
  if (socket?.readyState === WebSocket.OPEN) return
  socket = new WebSocket(url)

  const bootstrap_json = JSON.stringify({
    token: auth.accessToken,
    session_id: auth.session_id
  })

  socket.onopen = () => {
    if (isAuthed) socket!.send(`BOOTSTRAP:${bootstrap_json}`)
  }

  socket.onmessage = (e: MessageEvent) => {
    if (e.data === "Channel Created") return
    try {
      const msg = JSON.parse(e.data) as IncomingMessage
      if (!msg.namespace) return // drop anything without a namespace
      handleMessage(msg)
    } catch {
      // not json, ignore
    }
  }

  socket.onclose = () => {
    socket = null
    setTimeout(() => {
      addNotification({
        type: "important_info",
        title: "Notification",
        body: "Connection interrupted. Please reload the page to stay connected.",
        sender: "DevinLittle.Net",
        global: false,
      });
      /*
            if (auth.id) {
              refresh();
              setTimeout(connectNotifications, 3000)
            } else {
              setTimeout(connectNotifications, 3000)
            }
      
      */
      console.error("WEBSOCKET CLOSED");
    }, 1000)

  }

  socket.onerror = () => socket?.close()
}

export function disconnectNotifications() {
  socket?.close()
  socket = null
}

export async function sendMessage(msg: String, target_user_id: String) {
  await authFetch(`${API_URL}/notification/user_message/${target_user_id}`, {
    method: "POST",
    body: msg
  })
}
