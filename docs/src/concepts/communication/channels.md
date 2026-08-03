# Channels

A channel is a logical communication destination within the notification system.
> those destinations can be:
> 1) a global channel - a channel all users are subscribed to, authenticated or not
> 2) a special user id channel - only users with that User ID will receive the message
> 3) a admin/role channel - users with the admin role will only receive the message

Channels allow messages to be targeted at specific users, sessions, roles, or groups rather than broadcasting every message to every client.

Clients subscribe to channels when connecting to the notification backend, and events are delivered through those channels.
