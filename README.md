# The Repository powering Devin Little .Net

## WARNING, THIS IS BEING HEAVILY EDITED AND MAINTAINED RIGHT NOW
## EXPECT VERY BAD DOCUMENTATION FOR A WHILE WHILE THE PROJECT GROWS


DevinLittle.Net started out as a personal collection of tools built to improve my daily life. However, due to growing interest, I've refactored the ecosystem into a scalable platform capable of supporting a broader user base and rapidly deploying new applications/services.

## Repository Structure

This monorepo isolates my frontend, backend services, applications, and shared libraries. Click on any directory to view its specific documentation.

* [**`services/`**](./services) - the backend which is composed of microservices
    * `auth_backend/` - Authentication and User Data handleing
    * `gradegetter/` - the GradeGetter stack
    * `nanopass_backend/` - storing ephemeral nanopass metadata
    * `notification_backend/` - pub/sub service
    * `smalltalk_backend/` - notes + messaging
    * `backend-common/` - the common library that all services depend on
    * `service_connector/` - the WebRTC signalling server for the internal mesh network
* [**`frontend/website/`**](./frontend/website) - The web application built with Svelte
* [**`crates/`**](./crates) - Pure Rust utility libraries shared across the backend services
    * [`crypto_utils/`](./crates/crypto_utils) - encryption and decryption helpers
    * [`friendly-namer/`](./crates/friendly-namer/) - alias generation used by the internal mesh network
* [**`chart/`**](./chart) - deployment configurations
