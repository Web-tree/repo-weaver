#!/usr/bin/env sh
set -eu

env_name="$1"
region="$2"

cat > deploy.log <<LOG
Deploying payments service
Environment: ${env_name}
Region: ${region}
LOG
