variable "REGISTRY" { default = "git.home.devinlittle.net/devin" }
variable "TAG" { default = "dev" }
variable "GIT_COMMIT_HASH" {
  default = "unknown"
}

group "default" {
    targets = ["auth_backend", "gradegetter", "gradegetter_backend", "nanopass_backend", "notification_backend", "service_connector", "face_backend"]
}

target "settings" {
    context = ".."
    output = ["type=registry"]
    args = {
      GIT_COMMIT_HASH = GIT_COMMIT_HASH
    }
}

target "auth_backend" {
    inherits = ["settings"]
    dockerfile = "./services/auth_backend/Dockerfile"
    tags = ["${REGISTRY}/auth_backend:${TAG}"]
}

target "gradegetter" {
    inherits = ["settings"]
    dockerfile = "./services/gradegetter/Dockerfile.gradegetter"
    tags = ["${REGISTRY}/gradegetter:${TAG}"]
}

target "gradegetter_backend" {
    inherits = ["settings"]
    dockerfile = "./services/gradegetter/Dockerfile.gradegetter_backend"
    tags = ["${REGISTRY}/gradegetter_backend:${TAG}"]
}

target "nanopass_backend" {
    inherits = ["settings"]
    dockerfile = "./services/nanopass_backend/Dockerfile"
    tags = ["${REGISTRY}/nanopass_backend:${TAG}"]
}

target "notification_backend" {
    inherits = ["settings"]
    dockerfile = "./services/notification_backend/Dockerfile"
    tags = ["${REGISTRY}/notification_backend:${TAG}"]
}

target "smalltalk_backend" {
    inherits = ["settings"]
    dockerfile = "./services/smalltalk_backend/Dockerfile"
    tags = ["${REGISTRY}/smalltalk_backend:${TAG}"]
}

target "service_connector" {
    inherits = ["settings"]
    dockerfile = "./services/service_connector/Dockerfile"
    tags = ["${REGISTRY}/service_connector:${TAG}"]
}

target "face_backend" {
    inherits = ["settings"]
    dockerfile = "./services/face_backend/Dockerfile"
    tags = ["${REGISTRY}/service_connector:${TAG}"]
}
