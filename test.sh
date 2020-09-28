#!/bin/bash
set -e

COUCHDB_USER=admin
COUCHDB_PASSWORD=password

docker_id=$(docker run -d --rm -p 5984:5984 -e COUCHDB_USER=$COUCHDB_USER -e COUCHDB_PASSWORD=$COUCHDB_PASSWORD couchdb:3)

# docker kill if test is interrupted
trap 'docker kill $docker_id' SIGINT err exit

echo "Waiting for docker couchdb to be up..."
while ! curl -s -u $COUCHDB_USER:$COUCHDB_PASSWORD http://localhost:5984/; do
  sleep 0.1 # wait for 1/10 of the second before check again
done

echo "CouchDB is up. Starting tests in 1s"
sleep 1

cargo test -- --test-threads=1

docker kill "$docker_id"
