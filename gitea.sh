#!/bin/bash
echo 'version: "3"'
echo 'services:'

for count in {0..100}
do
	port=$(expr 11000 + $count)
	echo "  server_$count:"
	echo "    image: realaravinth/starchart-mock-gitea:latest"
	echo "    container_name: $count.mock_gitea"
	echo "    environment:"
	echo "      - USER_UID=1000"
	echo "      - USER_GID=1000"
	echo "      - PORT=$port"
	echo "      - DATABASE_URL=/var/log/mock-gitea"
	echo "      - MGITEA__DATA=/var/lib/unique.txt"
	echo "    restart: always"
	echo "    ports:"
	echo "     - '$port:$port'"
done
