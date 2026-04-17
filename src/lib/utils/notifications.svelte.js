import { auth } from "./auth.svelte.js";
import { API_URL } from "./auth.svelte.js";

export let notifications = $state([]);
let socket = null;


let dismissFn = null;

export function registerDismiss(fn) {
  dismissFn = fn;
}

function addNotification(notification) {
  const id = crypto.randomUUID();
  const n = { id, ...notification };
  notifications.push(n);
  if (n.type !== "offer") {
    setTimeout(() => {
      if (dismissFn) dismissFn(id);
    }, 5000);
  }
}

export function removeNotification(id) {
  const idx = notifications.findIndex(n => n.id === id);
  if (idx !== -1) notifications.splice(idx, 1);
}

export function connectNotifications() {
  const isAuthed = auth.id !== null;
  const path = isAuthed ? `/ws/${auth.id}` : `/ws/global`;
  const url = `${API_URL.replace("https://", "wss://").replace("http://", "ws://")}/notification${path}`;

  if (socket?.readyState === WebSocket.OPEN) return;
  socket = new WebSocket(url);

  socket.onopen = () => {
    if (isAuthed) socket.send(`BOOTSTRAP:${auth.accessToken}`);
  };

  socket.onmessage = (e) => {
    if (e.data === "Channel Created") return;
    try {
      const msg = JSON.parse(e.data);
      // offer type; manual dismiss, targeted to this session
      /*      if (msg.type === "offer") {
              if (msg.target_session_id !== auth.session_id) return;
              addNotification({
                type: "offer",
                title: "NanoPass Request",
                body: `${msg.sender_username} wants to send you a file`,
                sender: msg.sender_username,
                global: false,
                data: msg,
              });
              return;
            } */
      // global type; red, auto dismiss
      if (msg.type === "global") {
        addNotification({
          type: "global",
          title: msg.title ?? "Global",
          body: msg.content,
          global: true,
        });
        return;
      }
      // standard notification
      addNotification({
        type: "info",
        title: msg.title ?? "Notification",
        body: msg.content,
        sender: msg.sender_username ?? null,
        global: false,
      });
    } catch {
      // not json, ignore
    }
  };

  socket.onclose = () => {
    socket = null;
    // reconnect after 3s if auth still valid
    if (auth.id) setTimeout(connectNotifications, 3000);
  };

  socket.onerror = () => socket?.close();
}

export function disconnectNotifications() {
  socket?.close();
  socket = null;
}
