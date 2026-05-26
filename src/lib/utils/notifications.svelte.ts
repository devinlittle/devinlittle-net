import { auth, createClient, refresh } from "./auth.svelte";
import { fetchGrades } from "./gradegetter.svelte";
import { handleNanoPass, type NanoPassMessage } from "./nanopass.svelte";
import { handleKeySync, handleSmallTalkNotes, type SmallTalkNoteMessage } from "./smalltalk.svelte";
import type { KeySyncMessage } from "../types/smalltalk.types";
import type { components, paths as NotificationPaths } from "$lib/types/notification.api.ts";
import { API_URL } from "./constants.svelte";

export const notificationApi = createClient<NotificationPaths>(`${API_URL}/notification`);

// --- types ---
type Notification =
  {
    id: string
    type: 'info' | 'global' | "transfer" | "important_info" | "fast_info"
    title: string
    body: string
    global?: boolean
    sender?: string | null
    filename?: string
    filesize?: string
    dismissTime?: number
    onAccept?: () => void
    onDecline?: () => void
  }

type MessageNamespace = components["schemas"]["Namespaces"]

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

  if (n.type === "fast_info") {
    setTimeout(() => {
      if (dismissFn) dismissFn(id)
    }, n.dismissTime || 2000)

  }

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
  if (document.visibilityState === 'hidden') {
    new Notification(`${payload.title} ${payload.sender_username ? `- ${payload.sender_username}` : ``}`, {
      body: payload.content,
      icon: '/favicon.png'
    })
    return
  }

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
    case "smalltalk_notes":
      handleSmallTalkNotes(msg as SmallTalkNoteMessage)
      break
    case "smalltalk_keysync":
      handleKeySync(msg as KeySyncMessage)
      break
  }
}

let socketState = $state<"connected" | "connecting" | "disconnected">("disconnected");

function updateSocketState(newStatus: "connected" | "connecting" | "disconnected") {
  socketState = newStatus;

  switch (newStatus) {
    case "disconnected":
      console.error("disconnected");
      break;
    case "connecting":
      console.log("connecting");
      break;
    case "connected":
      console.log("connected to websocket");
      addNotification({
        type: "fast_info",
        title: "Notification",
        body: "connected to live session successfully",
        sender: "DevinLittle.Net",
        dismissTime: 750,
        global: false,
      });
      break;
  }
}


export function connectNotifications() {
  const isAuthed = auth.id !== null
  const path = isAuthed ? `/ws/${auth.id}` : `/ws/global`
  const url = `${API_URL.replace("https://", "wss://").replace("http://", "ws://")}/notification${path}`
  if (socket?.readyState === WebSocket.OPEN) return
  socket = new WebSocket(url)

  if (socket?.readyState === WebSocket.CONNECTING) {
    updateSocketState("connecting");
  }

  const bootstrap_json = JSON.stringify({
    token: auth.accessToken,
    session_id: auth.session_id
  })

  socket.onopen = () => {
    if (isAuthed) socket!.send(`BOOTSTRAP:${bootstrap_json}`)
    updateSocketState("connected");
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
    updateSocketState("disconnected");
    setTimeout(() => {
      // TODO: add button to notification to reconnect
      addNotification({
        type: "fast_info",
        title: "Notification",
        body: "Connection interrupted. Please reload the page to stay connected.",
        sender: "DevinLittle.Net",
        dismissTime: 5000,
        global: false,
      });
      console.error("WEBSOCKET CLOSED");
    }, 1000)

  }

  socket.onerror = () => socket?.close()
}

export function getSocket() {
  return socket;
}

export function disconnectNotifications() {
  socket?.close();
  socket = null
  updateSocketState("disconnected");
}

export async function sendMessage(msg: string, target_user_id: string) {
  // HACK: using fetch here is super hacky and a temp fix
  const url = `${API_URL}/notification/user_message/${target_user_id}`;

  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'text/plain',
      'Authorization': `Bearer ${auth.accessToken}`
    },
    body: msg,
  });

  if (!response.ok) {
    if (response.status === 401 || response.status === 403) {
      refresh();
      sendMessage(msg, target_user_id);
    }

  }

  //const user_message = notificationApi.path("/user_message/{id}").method("post").create() as any;
  //  await user_message({ id: target_user_id }, msg);
}
