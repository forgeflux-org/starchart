#!/bin/bash
echo 'version: "3"'
echo 'services:'

for count in {0..100}
do
	port=$(expr 11000 + $count)
	echo "  server_$count:"
	echo "    image: realaravinth/starchart-mock-gitea:latest"
	echo "    container_name: $count.mock_gitea.starchart.test.forgeflux.org"
	echo "    environment:"
	echo "      - USER_UID=1000"
	echo "      - USER_GID=1000"
	echo "      - DATABASE_URL=/var/log/mock-gitea"
	echo "      - MGITEA__DATA=/var/lib/unique.txt"
	echo "    restart: always"
	#echo "    networks:"
	#echo "      - gitea"
	echo "    ports:"
	echo "     - '$port:3000'"
done
