#!/bin/bash

# To be run on the ec2 instance, just copy-paste it into a bash script or line by line

sudo dnf install git gcc;

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh;

git clone https://github.com/Will-Banksy/done_server.git;

cd done_server;

cargo build;

echo "Instance set up and server compiled, be sure to create the .env file before running the server";