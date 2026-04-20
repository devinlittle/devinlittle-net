import { auth } from "./auth.svelte.js"
import type { NanoPassMessage, NanoPassPayload, FileListing } from "./nanopass.types.js"

// --- state ---

export let listings = $state<FileListing[]>([])

// --- entry point called from notifications.svelte.ts ---

export function handleNanoPass(msg: NanoPassMessage) {
  // drop if WE sent this AND it was targeted broadcast
  if (msg.from_session_id === auth.session_id && msg.target_session_id !== null) return

  // drop targeted messages not meant for this device
  if (msg.target_session_id !== null && msg.target_session_id !== auth.session_id) return

  const payload = msg.payload
  switch (payload.type) {
    case "ListingAdded": return handleListingAdded(payload.listing)
    case "ListingRemoved": return handleListingRemoved(payload.listing_id)
    case "FileQuery": return handleFileQuery(msg)
    case "FileQueryResponse": return handleFileQueryResponse(msg)
    case "TransferRequest": return handleTransferRequest(msg)
    case "TransferAccepted": return handleTransferAccepted(msg)
    case "TransferDeclined": return handleTransferDeclined(msg)
    case "SDPOffer": return handleSDPOffer(msg)
    case "SDPAnswer": return handleSDPAnswer(msg)
    case "ICECandidate": return handleICECandidate(msg)
  }
}

// --- listings ---

function handleListingAdded(listing: FileListing) {
  if (listings.some(l => l.id === listing.id)) return
  listings.push(listing)
}

function handleListingRemoved(listing_id: string) {
  const idx = listings.findIndex(l => l.id === listing_id)
  if (idx !== -1) listings.splice(idx, 1)
}

// --- ARP resolution ---

function handleFileQuery(msg: NanoPassMessage) {
  // TODO: check if we are hosting listing_id, if so respond with FileQueryResponse
}

function handleFileQueryResponse(msg: NanoPassMessage) {
  // TODO: store resolved host_session_id for this listing_id
}

// --- transfer handshake ---

function handleTransferRequest(msg: NanoPassMessage) {
  // TODO: show FaceTime style notification
}

function handleTransferAccepted(msg: NanoPassMessage) {
  // TODO: begin WebRTC signaling
}

function handleTransferDeclined(msg: NanoPassMessage) {
  // TODO: notify user that transfer was declined
}

// --- webrtc signaling ---

function handleSDPOffer(msg: NanoPassMessage) {
  // TODO: setRemoteDescription, createAnswer, setLocalDescription, send SDPAnswer
}

function handleSDPAnswer(msg: NanoPassMessage) {
  // TODO: setRemoteDescription
}

function handleICECandidate(msg: NanoPassMessage) {
  // TODO: addIceCandidate
}
