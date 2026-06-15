# DevinLittle.net Services

### Service Port Matrix

| Service | Directory Name | Assigned Local Port | OpenAPI Spec (JSON) / Interactive Scalar Docs | Role / Core Responsibility |
| :--- | :--- | :--- | :--- | :--- |
| **Auth Service** | `auth_backend` | `:3000` | [OpenAPI JSON](https://api.devinlittle.net/auth/api-docs/openapi.json) <br> [Scalar Docs](https://api.devinlittle.net/auth/scalar) | Secure authentication and user data handling |
| **GradeGetter Worker** | `gradegetter` | `:3001` | *N/A* | Automated scraping and data extraction worker |
| **GradeGetter Backend** | `gradegetter_backend` | `:3002` | [OpenAPI JSON](https://api.devinlittle.net/gradegetter/api-docs/openapi.json) <br> [Scalar Docs](https://api.devinlittle.net/gradegetter/scalar) | Interface for the GradeGetter stack |
| **Notification Service** | `notification_backend` | `:3003` | [OpenAPI JSON](https://api.devinlittle.net/notification/api-docs/openapi.json) <br> [Scalar Docs](https://api.devinlittle.net/notification/scalar) | Asynchronous pub/sub event stream broker |
| **Nanopass** | `nanopass_backend` | `:3004` | [OpenAPI JSON](https://api.devinlittle.net/nanopass/api-docs/openapi.json) <br> [Scalar Docs](https://api.devinlittle.net/nanopass/scalar) | handling of ephemeral FileListing metadata |
| **Smalltalk** | `smalltalk_backend` | `:3005` | [OpenAPI JSON](https://api.devinlittle.net/smalltalk/api-docs/openapi.json) <br> [Scalar Docs](https://api.devinlittle.net/smalltalk/scalar) | Interface for notes and messaging, allowing delta syncs and real time messaging and notes |
| **Service Connector** | `service_connector` | `:3006` | *N/A* | WebRTC signaling server for the internal mesh network |
