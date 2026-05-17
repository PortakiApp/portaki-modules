variable "database_url" {
  type    = string
  default = getenv("DATABASE_URL")
}

env "local" {
  url = var.database_url
  migration {
    dir = "file://migrations"
  }
}
