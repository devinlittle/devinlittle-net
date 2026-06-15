# NanoPass

NanoPass is a p2p file sharing service. With its p2p nature, there isn't much logic existing on the server, most exists on the client. However, NanoPass_Backend exists for a reason so i'll just go up and at em and explain this:

NanoPass backend exposes an interface in which clients use to Create, Modify, or Delete FileListings. FileListings contain metadata that clients use in order to signal and communicate exactly what file they want. NanoPass File Listings are designed to be epemeral and not live on beyond a sessions life span.
