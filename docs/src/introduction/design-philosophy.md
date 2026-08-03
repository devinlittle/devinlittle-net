# Goals and Design Philosophy

I've designed DLN around a few core principles:

---
## Shared Infrastructure
I essentially wanted a google workspace of my own. I wanted to create a place where I have 1 login and all my services are connected. Instead of creating a bunch of 1 off tools, I have 1 centralized place for all my services.

## Event-Driven (the live feeling)
I love when applications have this feeling of being alive and I also love the user experience of not needing to refresh in order to get up to date data, so I built these applications with my Event-Driven ideals in mind.

## Lightweight
I wanted both clients and servers to be very light.

The whole server stack can run on a Raspberry Pi Zero 2 along with a client.

Clients can run on anything, and my descision to choose Qt can enable clients to sit in the background of your OS without offering much overhead in terms of CPU and RAM usage (yk with the whole ram situation its key that dln-ui doesnt just suckup 5 Gibs of ram lol)

## Platform Consistency
I wanted to offer many platforms, so keeping data flow consistent is required and something to boast about too.

## Self-Hosted First
I run many self hosted applications on my own homelab and if people dont like having all their encrypted notes on my databases then they can deploy their own DLN and just change the api_url in both the frontend and clients to use their Infrastructure rather than my own.
