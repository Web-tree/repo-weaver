terraform {
  required_version = ">= 1.7.0"
}

provider "aws" {
  region = "us-east-1"
}

resource "aws_instance" "app" {
  ami           = "ami-0a12345b67890cdef"
  instance_type = "t3.micro"

  tags = {
    Name = "orders-api"
  }
}
