import { auth, API_URL, refresh } from "./auth.svelte.js";
import { fetchGrades } from "./gradegetter.svelte.js";

// --- types ---

type NotificationType = "global" | "info" | "offer"

type Notification = {
  id: string
  type: NotificationType
  title: string
  body: string
  global?: boolean
  sender?: string | null
  data?: unknown
}

type MessageNamespace = "notification" | "nanopass" | "gradegetter"

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

function addNotification(notification: Omit<Notification, "id">) {
  const id = crypto.randomUUID()
  const n: Notification = { id, ...notification }
  notifications.push(n)
  if (n.type !== "offer") {
    setTimeout(() => {
      if (dismissFn) dismissFn(id)
    }, 5000)
  }
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

function handleNanoPass(payload: unknown) {
  // TODO: implement NanoPass protocol logic
}

// --- socket ---

function handleMessage(msg: IncomingMessage) {
  switch (msg.namespace) {
    case "notification":
      handleNotification(msg.payload as NotificationPayload)
      break
    case "nanopass":
      handleNanoPass(msg.payload)
      break
    case "gradegetter":
      fetchGrades();
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
    console.error("WEBSOCKET CLOSED");
    if (auth.id) {
      setTimeout(connectNotifications, 3000)
      refresh();
    }
  }

  socket.onerror = () => socket?.close()
}

export function disconnectNotifications() {
  socket?.close()
  socket = null
}
