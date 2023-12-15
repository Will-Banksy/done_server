#!/bin/bash

# To be run on the ec2 instance, just copy-paste it into a bash script or line by line into the terminal

# WARNING: Minimal EC2 instances with 1 GB RAM won't be able to compile the project, and will fail at the linking stage

sudo dnf install git gcc;

# Skip this step if not compiling project from source
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh;

git clone https://github.com/Will-Banksy/done_server.git;

cd done_server;

# Skip this step if not compiling project from source
cargo build;

# Optional steps to download the (as of writing) latest version of Caddy
wget "https://github.com/caddyserver/caddy/releases/download/v2.7.6/caddy_2.7.6_linux_amd64.tar.gz"
tar -xf caddy_2.7.6_linux_amd64.tar.gz

echo "Instance set up and server compiled, be sure to create the .env file before running the server";