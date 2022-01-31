variable "container_registry" {
  type        = string
  default     = "localhost:5000"
  description = "The container registry in which we can find Grapl services."
}

variable "plugin_id" {
  type        = string
  description = "The ID for this plugin."
}

variable "tenant_id" {
  type        = string
  description = "The tenant's ID. Used in the plugin-execution-sidecar."
}

variable "plugin_artifact_url" {
  type        = string
  description = "The url that specifies which binary to run as the plugin."
}

variable "plugin_count" {
  type        = number
  default     = 1
  description = "The number of instances of the plugin to run."
}

variable "aws_account_id" {
  type        = string
  description = "The account ID of the aws account that holds onto the plugin binaries."
}

# Temporarily dropping the shared_key stuff and picking it up later, per
# https://github.com/grapl-security/grapl/pull/1403
# locals {
#   shared_key = "grapl_secret-${uuidv4()}"
# }

job "grapl-plugin" {
  datacenters = ["dc1"]
  namespace   = "plugin-${var.plugin_id}"
  type        = "service"

  # We'll want to make sure we have the opposite constraint on other services
  # This is set in the Nomad agent's `client` stanza:
  # https://www.nomadproject.io/docs/configuration/client#meta
  constraint {
    attribute = "${meta.is_grapl_plugin_host}"
    value     = true
  }

  group "plugin" {
    network {
      port "plugin_grpc_receiver" {}
    }

    restart {
      attempts = 1
    }

    count = var.plugin_count

    task "tenant-plugin-execution-sidecar" {
      driver = "docker"

      template {
        data        = <<EOH
      {{ $plugin_id := env "PLUGIN_ID" }}
      {{ with secret "pki/issue/plugin_execution_ca" (printf "common_name=%s.plugins.grapl.com" $plugin_id) "format=pem" }}
        {{ .Data.certificate }}
        {{ .Data.issuing_ca }}
        {{ .Data.private_key }}
      {{ end }}
      EOH
        destination = "${NOMAD_SECRETS_DIR}/bundle.pem"
        change_mode = "restart"
      }

      config {
        image = "grapl/plugin-execution-sidecar"
        ports = [
        "plugin_sidecar_grpc_receiver"]
      }

      env {
        TENANT_ID = "${var.tenant_id}"
        PLUGIN_ID = "${var.plugin_id}"
        # Temporarily dropping shared_key stuff
        # BOOTSTRAP_KEY = "${local.shared_key}"
      }
    }


    task "tenant-plugin-bootstrap-sidecar" {
      driver = "docker"

      template {
        data        = <<EOH
      {{ $plugin_id := env "PLUGIN_ID" }}
      {{ with secret "pki/issue/plugin_bootstrap_ca" (printf "common_name=%s.plugins.grapl.com" $plugin_id) "format=pem" }}
        {{ .Data.certificate }}
        {{ .Data.issuing_ca }}
        {{ .Data.private_key }}
      {{ end }}
      EOH
        destination = "${NOMAD_SECRETS_DIR}/bundle.pem"
        change_mode = "restart"
      }

      config {
        image = "grapl/plugin-bootstrap-sidecar"
        ports = [
        "plugin_bootstrap_grpc_receiver"]
      }

      env {
        TENANT_ID = "${var.tenant_id}"
        PLUGIN_ID = "${var.plugin_id}"
        # Temporarily dropping shared_key stuff
        # BOOTSTRAP_KEY = "${local.shared_key}"
      }
    }


    task "tenant-plugin" {
      driver = "firecracker-task-driver"

      artifact {
        source      = var.plugin_artifact_url
        destination = "local/plugin"
        mode        = "file"
        headers {
          x-amz-expected-bucket-owner = var.aws_account_id
          x-amz-meta-client-id        = "nomad-deployer"
        }
      }

      artifact {
        source      = "https://grapl-firecracker.s3.amazonaws.com/kernel/v0.tar.gz"
        destination = "local/vmlinux"
        headers {
          x-amz-expected-bucket-owner = var.aws_account_id
          x-amz-meta-client-id        = "nomad-deployer"
        }
      }

      artifact {
        source      = "https://grapl-firecracker.s3.amazonaws.com/rootfs/v0.rootfs.tar.gz"
        destination = "local/rootfs.ext4"
        headers {
          x-amz-expected-bucket-owner = var.aws_account_id
          x-amz-meta-client-id        = "nomad-deployer"
        }
      }

      config {
        KernelImage = "local/vmlinux"
        BootDisk    = "local/rootfs.ext4"
        //        Disks = [ "local/plugin" ]
        Firecracker = "/usr/bin/firecracker"
        Vcpus       = 1
        Mem         = 128
        Network     = "default"
      }

      service {
        name = "plugin-${var.plugin_id}"
        port = "plugin_grpc_receiver"
        tags = [
          "plugin",
          "tenant-${var.tenant_id}",
          "plugin-${var.plugin_id}"
        ]

        # https://www.nomadproject.io/docs/job-specification/service#grpc-health-check
        check {
          type         = "grpc"
          port         = "plugin_grpc_receiver"
          interval     = "4s"
          timeout      = "1s"
          grpc_service = "Health.Service"
          grpc_use_tls = true
        }

      }
    }
  }
}