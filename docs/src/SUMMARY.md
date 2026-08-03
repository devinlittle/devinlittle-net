# Summary

# Introduction

* [What is DLN?](introduction/what-is-dln.md)
* [Goals and Design Philosophy](introduction/design-philosophy.md)
* [Repository Structure](introduction/repository-structure.md)

# Concepts

* [Identity](concepts/identity.md)

  * [User](concepts/identity/user.md)
  * [User ID](concepts/identity/user-id.md)
  * [Session](concepts/identity/session.md)
  * [Session ID](concepts/identity/session-id.md)
  * [Access Tokens](concepts/identity/access-tokens.md)
  * [Refresh Tokens](concepts/identity/refresh-tokens.md)
  * [Roles](concepts/identity/roles.md)

* [Communication](concepts/communication.md)

  * [Channels](concepts/communication/channels.md)
  * [Events (Messages)](concepts/communication/events.md)
  * [Namespaces](concepts/communication/namespaces.md)
  * [Routing](concepts/communication/routing.md)

* [Presence](concepts/presence.md)

* [Trust](concepts/trust.md)

* [State](concepts/state.md)

* [Clients](concepts/clients.md)

* [Services](concepts/services.md)

# Protocols

* [Authentication](protocols/authentication.md)
* [Token Refresh](protocols/token-refresh.md)
* [Notification Bootstrap](protocols/notification-bootstrap.md)
* [Event Delivery](protocols/event-delivery.md)
* [Presence Tracking](protocols/presence-tracking.md)
* [Nanopass Discovery](protocols/nanopass-discovery.md)
* [Key Synchronization](protocols/key-synchronization.md)
* [Mesh Discovery](protocols/mesh-discovery.md)

# Architecture

* [System Overview](architecture/system-overview.md)
* [Identity Architecture](architecture/identity-architecture.md)
* [Client Architecture](architecture/client-architecture.md)
* [Notification Architecture](architecture/notification-architecture.md)
* [Service Architecture](architecture/service-architecture.md)
* [Data Flow](architecture/data-flow.md)
* [Deployment Architecture](architecture/deployment-architecture.md)

# Services

* [Auth](services/auth.md)
* [Notifications](services/notifications.md)
* [GradeGetter](services/gradegetter.md)
* [Nanopass](services/nanopass.md)
* [Smalltalk](services/smalltalk.md)
* [Service Connector](services/service-connector.md)

# Libraries

* [backend-common](libraries/backend-common.md)
* [dln-core](libraries/dln-core.md)
* [crypto-utils](libraries/crypto-utils.md)
* [friendly-namer](libraries/friendly-namer.md)

# Deployment

* [Overview](deployment/overview.md)
* [Bare Metal](deployment/bare-metal.md)  <!--Bare Metal kinda not viaable yet...-->
* [Docker Compose](deployment/docker-compose.md)
* [Kubernetes](deployment/kubernetes.md)
* [Environment Variables](deployment/environment-variables.md)
* [Reverse Proxies](deployment/reverse-proxies.md)
* [Scaling](deployment/scaling.md)  <!--Scaling kinda not viaable yet...-->

# Development

* [Workspace Layout](development/workspace-layout.md)
* [Building](development/building.md)
* [Testing](development/testing.md)
* [Release Process](development/release-process.md)

# Reference

* [Glossary](reference/glossary.md)
* [FAQ](reference/faq.md)
