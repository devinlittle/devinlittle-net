import { API_URL, auth, authFetch } from "./auth.svelte.js"
import type { NanoPassMessage, NanoPassPayload, FileListing } from "./nanopass.types.js"
import { addNotification, formatBytes, sendMessage } from "./notifications.svelte.js"

// --- state ---

export const nanopass = $state({
  listings: [] as FileListing[],
  transferProgress: {} as Record<string, number>
})

// --- entry point called from notifications.svelte.ts ---

export function handleNanoPass(msg: NanoPassMessage) {
  console.log(msg.payload.type, '| from:', msg.from_session_id, '| me:', auth.session_id, '| match:', msg.from_session_id === auth.session_id, '| target:', msg.target_session_id)
  // drop if WE sent this AND it was targeted
  if (msg.from_session_id === auth.session_id && msg.target_session_id !== null) return

  // drop targeted messages not meant for this device
  if (msg.target_session_id !== null && msg.target_session_id !== auth.session_id) return

  const payload = msg.payload
  switch (payload.type) {
    case "ListingAdded": return handleListingAdded(payload.listing)
    case "ListingRemoved": return handleListingRemoved(payload.listing.id)
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
  if (nanopass.listings.some(l => l.id === listing.id)) return
  nanopass.listings.push(listing)
}


function handleListingRemoved(listing_id: string) {
  const idx = nanopass.listings.findIndex(l => l.id == listing_id)
  if (idx !== -1) nanopass.listings.splice(idx, 1)
}

// --- ARP resolution ---

function handleFileQuery(msg: NanoPassMessage) {
  const payload = msg.payload as Extract<NanoPassPayload, { type: 'FileQuery' }>
  const hosted = hostedFiles.get(payload.listing_id)
  if (!hosted) return // not our file; ignore
  sendNanoPass({
    type: 'FileQueryResponse',
    listing_id: payload.listing_id,
    host_session_id: auth.session_id!
  }, msg.from_session_id, msg.from_user_id)

}

const resolvedHosts = new Map<string, string>()

function handleFileQueryResponse(msg: NanoPassMessage) {
  // TODO: store resolved host_session_id for this listing_id
  const payload = msg.payload as Extract<NanoPassPayload, { type: 'FileQueryResponse' }>
  resolvedHosts.set(payload.listing_id, payload.host_session_id)
  // now we know who to send the TransferRequest to
  sendNanoPass({
    type: 'TransferRequest',
    listing_id: payload.listing_id,
    requester_session_id: auth.session_id!
  }, payload.host_session_id, msg.from_user_id)
}

// --- transfer handshake ---

// this is the facetime like notifiation
function handleTransferRequest(msg: NanoPassMessage) {
  const payload = msg.payload as Extract<NanoPassPayload, { type: 'TransferRequest' }>
  const listing = nanopass.listings.find(l => l.id === payload.listing_id)
  if (!listing) return
  addTransferNotification({
    listing,
    requester_session_id: payload.requester_session_id,
    onAccept: () => {
      sendNanoPass({ type: 'TransferAccepted', listing_id: payload.listing_id }, msg.from_session_id, msg.from_user_id)
      // host creates offer
      initWebRTCAsHost(payload.listing_id, msg.from_session_id, msg.from_user_id)
    },
    onDecline: () => {
      sendNanoPass({ type: 'TransferDeclined', listing_id: payload.listing_id }, msg.from_session_id, msg.from_user_id)
    }
  })
}

function handleTransferAccepted(msg: NanoPassMessage) {
  const payload = msg.payload as Extract<NanoPassPayload, { type: 'TransferAccepted' }>
  initWebRTCAsRequester(payload.listing_id, msg.from_session_id, msg.from_user_id)
}

function handleTransferDeclined(msg: NanoPassMessage) {
  const payload = msg.payload as Extract<NanoPassPayload, { type: 'TransferDeclined' }>
  console.log('transfer declined for', payload.listing_id)
}

function addTransferNotification({ listing, requester_session_id, onAccept, onDecline }) {
  addNotification({
    type: 'transfer',
    title: 'incoming file request',
    body: `a device wants to download from you`,
    filename: listing.filename,
    filesize: formatBytes(listing.size_bytes),
    onAccept,
    onDecline,
  })
}

// --- webrtc signaling ---


// setRemoteDescription, createAnswer, setLocalDescription, send SDPAnswer
async function handleSDPOffer(msg: NanoPassMessage) {
  const payload = msg.payload as Extract<NanoPassPayload, { type: 'SDPOffer' }>
  console.log('handleSDPOffer, pc exists:', !!peerConnections.get(payload.listing_id))
  const pc = peerConnections.get(payload.listing_id)
  if (!pc) {
    console.error('NO PC FOUND for', payload.listing_id)
    return
  }
  await pc.setRemoteDescription({ type: 'offer', sdp: payload.sdp })
  const answer = await pc.createAnswer()
  await pc.setLocalDescription(answer)  // this should trigger ICE gathering
  console.log('gathering state after setLocalDescription:', pc.iceGatheringState)
  sendNanoPass({ type: 'SDPAnswer', listing_id: payload.listing_id, sdp: answer.sdp! }, msg.from_session_id, msg.from_user_id)
}

// setRemoteDescription
async function handleSDPAnswer(msg: NanoPassMessage) {
  const payload = msg.payload as Extract<NanoPassPayload, { type: 'SDPAnswer' }>
  const pc = peerConnections.get(payload.listing_id)
  if (!pc) return
  await pc.setRemoteDescription({ type: 'answer', sdp: payload.sdp })
}

// addIceCandidate
async function handleICECandidate(msg: NanoPassMessage) {
  const payload = msg.payload as Extract<NanoPassPayload, { type: 'ICECandidate' }>
  const pc = peerConnections.get(payload.listing_id)
  if (!pc) return
  await pc.addIceCandidate({
    candidate: payload.candidate,
    sdpMid: payload.sdp_mid ?? undefined,
    sdpMLineIndex: payload.sdp_mline_index ?? undefined
  })

}

const peerConnections = new Map<string, RTCPeerConnection>()

function createPeerConnection(listing_id: string, target_session_id: string, target_user_id: String): RTCPeerConnection {
  const pc = new RTCPeerConnection({
    iceServers: [
      { urls: "stun:turn.devinlittle.net:3478" },
      {
        urls: 'turn:turn.devinlittle.net:3478',
        username: 'devin',
        credential: 'little'
      },
    ]
  })
  peerConnections.set(listing_id, pc)
  pc.onicecandidate = (e) => {
    console.log('ICE candidate:', e.candidate)
    if (!e.candidate) return
    sendNanoPass({
      type: 'ICECandidate',
      listing_id,
      candidate: e.candidate.candidate,
      sdp_mid: e.candidate.sdpMid ?? null,
      sdp_mline_index: e.candidate.sdpMLineIndex ?? null
    }, target_session_id, target_user_id)
  }

  pc.oniceconnectionstatechange = () => {
    console.log('ICE state:', pc.iceConnectionState)
  }


  return pc
}



async function initWebRTCAsRequester(listing_id: string, target_session_id: string, target_user_id: String) {
  const pc = createPeerConnection(listing_id, target_session_id, target_user_id)
  const dc = pc.createDataChannel('filetransfer')
  receiveFileInChunks(dc, listing_id)
  const offer = await pc.createOffer()
  await pc.setLocalDescription(offer)
  sendNanoPass({ type: 'SDPOffer', listing_id, sdp: offer.sdp! }, target_session_id, target_user_id)
}

function initWebRTCAsHost(listing_id: string, target_session_id: string, target_user_id: String) {
  const pc = createPeerConnection(listing_id, target_session_id, target_user_id)
  pc.ondatachannel = (e) => {
    e.channel.onopen = () => sendFileInChunks(listing_id, e.channel)
  }
}

// ---- data fetch/send functions ----

export async function fetchListings() {
  const response = await authFetch(`${API_URL}/nanopass/listings`)
  if (!response.ok) return
  const data: FileListing[] = await response.json()
  nanopass.listings = data
}


export function sendNanoPass(payload: NanoPassPayload, target_session_id: string | null = null, target_user_id: String) {
  sendMessage(JSON.stringify({
    namespace: 'nanopass',
    id: crypto.randomUUID(),
    from_session_id: auth.session_id,
    from_user_id: auth.id,
    target_user_id,
    target_session_id,
    payload
  }), target_user_id)
}

// --- file stuffies ---

export const hostedFiles = new Map<string, File>()

export function registerHostedFile(listing_id: string, file: File) {
  hostedFiles.set(listing_id, file)
}

const CHUNK_SIZE = 256 * 1024             // 256KB
const HIGH_WATER_MARK = 8 * 1024 * 1024  // 8MB, buffer ceiling
const LOW_WATER_MARK = 2 * 1024 * 1024  // 2MB, resume threshold

export function sendFileInChunks(listing_id: string, dc: RTCDataChannel) {
  const file = hostedFiles.get(listing_id);
  if (!file) return;

  dc.binaryType = 'arraybuffer';
  dc.bufferedAmountLowThreshold = LOW_WATER_MARK;

  const totalChunks = Math.ceil(file.size / CHUNK_SIZE);
  let index = 0;
  let paused = false;

  async function sendNext() {
    while (index < totalChunks) {
      if (dc.bufferedAmount >= HIGH_WATER_MARK) {
        paused = true;
        return;
      }

      const offset = index * CHUNK_SIZE;
      const buffer = await file.slice(offset, offset + CHUNK_SIZE).arrayBuffer();

      dc.send(JSON.stringify({
        index,
        total: totalChunks,
        filename: file.name,
        mime_type: file.type || 'application/octet-stream',
      }));

      dc.send(buffer);
      index++;
    }

    const finalCheck = setInterval(() => {
      if (dc.bufferedAmount === 0) {
        clearInterval(finalCheck);
        dc.send(JSON.stringify({ done: true }));
        console.log("Sender: All bytes cleared from buffer and sent.");
      }
    }, 100);
  }

  dc.onbufferedamountlow = () => {
    if (paused) {
      paused = false;
      sendNext();
    }
  };

  sendNext();
}

export function receiveFileInChunks(dc: RTCDataChannel, listing_id: string) {
  let chunks: ArrayBuffer[] = [];
  let pendingHeader: { index: number; total: number; filename: string; mime_type: string } | null = null;
  let filename = '';
  let mimeType = '';
  let total = 0;
  let receivedCount = 0;

  dc.binaryType = 'arraybuffer';

  dc.onmessage = (e) => {
    if (typeof e.data === 'string') {
      const msg = JSON.parse(e.data);

      if (msg.done) {
        const blob = new Blob(chunks, { type: mimeType });

        if (blob.size === 0) {
          console.error("Blob is empty! Check chunk sequence.");
          return;
        }

        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();


        setTimeout(() => {
          document.body.removeChild(a);
          URL.revokeObjectURL(url);
        }, 2000);

        nanopass.transferProgress[listing_id] = 1;
        return;
      }
      pendingHeader = msg;
      filename = msg.filename;
      mimeType = msg.mime_type;
      total = msg.total;
      if (chunks.length === 0) {
        chunks = new Array(total);
      }
    } else {
      if (!pendingHeader) return;
      chunks[pendingHeader.index] = e.data as ArrayBuffer;
      receivedCount++;
      pendingHeader = null;
      nanopass.transferProgress[listing_id] = receivedCount / total;
    }
  };
}
