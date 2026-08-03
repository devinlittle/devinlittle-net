//variable "REGISTRY" { default = "10.10.10.245:5000/devin" }
variable "REGISTRY" { default = "git.home.devinlittle.net/devin" }
variable "TAG" { default = "dev" }

group "default" {
    targets = ["docs"]
}

target "docs" {
    context = "."
    dockerfile = "Dockerfile"
    tags = ["${REGISTRY}/devinlittle-net_docs:${TAG}"]
    output = ["type=registry"]
}
