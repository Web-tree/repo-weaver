variable "cluster_name" {
  type = string
}

resource "null_resource" "cluster" {
  provisioner "local-exec" {
    command = "echo Creating cluster ${var.cluster_name}"
  }
}
