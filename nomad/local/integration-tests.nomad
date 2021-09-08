# This setup is inspired by the following forum discussion:
# https://discuss.hashicorp.com/t/best-practices-for-testing-against-services-in-nomad-consul-connect/29022
# We'll submit integration tests to Nomad as 
# 
variable "container_registry" {
  type        = string
  default     = "localhost:5000"
  description = "The container registry in which we can find Grapl services."
}

variable "aws_region" {
  type = string
}

variable "deployment_name" {
  type        = string
  description = "The deployment name"
}

variable "aws_access_key_id" {
  type        = string
  default     = "test"
  description = "The aws access key id used to interact with AWS."
}

variable "aws_access_key_secret" {
  type        = string
  default     = "test"
  description = "The aws access key secret used to interact with AWS."
}

variable "aws_endpoint" {
  type        = string
  description = "The endpoint in which we can expect to find and interact with AWS."
}

variable "redis_endpoint" {
  type        = string
  description = "Where can services find redis?"
}

locals {
  log_level = "DEBUG"
}

job "integration-tests" {
  datacenters = ["dc1"]
  type        = "batch"
  parameterized {}

  reschedule {
    # Make this a one-shot job
    attempts = 0
  }

  # Specifies that this job is the most high priority job we have; nothing else should take precedence 
  priority = 100

  group "integration-tests" {
    restart {
      # Make this a one-shot job
      attempts = 0
    }

    network {
      mode = "bridge"
    }

    # Enable service discovery
    service {
      connect {
        sidecar_service {
          proxy {
            upstreams {
              # This is a hack, because IDK how to share locals across files
              destination_name = "dgraph-alpha-0-grpc-public"
              local_bind_port  = 9080
            }
          }
        }
      }
    }

    task "rust-integration-tests" {
      driver = "docker"

      config {
        image = "${var.container_registry}/grapl/rust-integration-tests:latest"
      }

      env {
        AWS_REGION                  = var.aws_region
        DEPLOYMENT_NAME             = var.deployment_name
        GRAPL_AWS_ENDPOINT          = var.aws_endpoint
        GRAPL_AWS_ACCESS_KEY_ID     = var.aws_access_key_id
        GRAPL_AWS_ACCESS_KEY_SECRET = var.aws_access_key_secret
        GRAPL_LOG_LEVEL             = local.log_level
        # This is a hack, because IDK how to share locals across files
        #MG_ALPHAS                   = local.alpha_grpc_connect_str # TODO: Figure out how to do this
        MG_ALPHAS      = "localhost:9080"
        RUST_BACKTRACE = 1
        RUST_LOG       = local.log_level
        REDIS_ENDPOINT = var.redis_endpoint
      }
    }
  }
}

