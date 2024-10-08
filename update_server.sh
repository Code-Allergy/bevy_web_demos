#!/usr/bin/env bash
# This does NOT build the app, it just updates the server with the current dist package.

SERVER_USER="root"
SERVER_IP="vada.life"
SERVER_PATH="/var/www/vada.life"

# Package the web app
tar -czf dist.tar.gz -C dist .

# Copy the package to the server
scp dist.tar.gz $SERVER_USER@$SERVER_IP:~
ssh $SERVER_USER@$SERVER_IP "rm -rf $SERVER_PATH && \
                             mkdir -p $SERVER_PATH && \
                             cd $SERVER_PATH && \
                             mv ~/dist.tar.gz . && \
                             tar -xzf dist.tar.gz && \
                             rm -f dist.tar.gz"

# Clean up
rm dist.tar.gz
