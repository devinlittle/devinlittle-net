// --- envelope ---

import type { FileListing } from "$lib/utils/nanopass.svelte"

export type NanoPassMessage = {
  id: string
  from_session_id: string
  from_user_id: string
  target_user_id: string
  target_session_id: string | null
  payload: NanoPassPayload
}

// --- payload ---

export type NanoPassPayload =
  | { type: "FileQuery"; listing_id: string; requester_session_id: string }
  | { type: "FileQueryResponse"; listing_id: string; host_session_id: string }
  | { type: "TransferRequest"; listing_id: string; requester_session_id: string, requester_username: string }
  | { type: "TransferAccepted"; listing_id: string }
  | { type: "TransferDeclined"; listing_id: string }
  | { type: "SDPOffer"; listing_id: string; sdp: string }
  | { type: "SDPAnswer"; listing_id: string; sdp: string }
  | { type: "ICECandidate"; listing_id: string; candidate: string; sdp_mid: string | null; sdp_mline_index: number | null }
  | { type: "ListingAdded"; listing: FileListing }
  | { type: "ListingModified"; listing: FileListing }
  | { type: "ListingRemoved"; listing: FileListing }
