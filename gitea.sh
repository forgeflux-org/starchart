#!/bin/bash
echo 'version: "3"'
echo 'services:'

for count in {0..100}
do
	port=$(expr 11000 + $count)
	docker rm -f server_$count
echo "  server_$count:"
echo "    image: gitea/gitea:1.16.5"
echo "    container_name: gitea_$count"
echo "    environment:"
echo "      - USER_UID=1000"
echo "      - USER_GID=1000"
echo "    restart: always"
#echo "    networks:"
#echo "      - gitea"
echo "    ports:"
echo "     - '$port:3000'"
done
