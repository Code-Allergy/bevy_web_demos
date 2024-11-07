#!/usr/bin/env bash
set -e # Exit on error
SERVER_USER="root"
SERVER_IP="vada.life"
SERVER_PATH="/var/www/vada.life"

# Build the app
make RELEASE=true

# Package the source code
./package.sh

# Move source packages into dist
mv target/source.zip dist
mv target/source.tar.gz dist

# Package the web app
tar -czvf dist.tar.gz -C dist .

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
