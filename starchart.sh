#!/bin/bash
echo 'version: "3"'
echo 'services:'

introducer=""
for count in {0..10}
do
	port=$(expr 12001 + $count)
	container_name="$count.starchart.test.forgeflux.org"
	echo "  $container_name:"
	echo "    image: realaravinth/starchart:latest"
	echo "    container_name: $container_name"
	echo "    environment:"
	echo "      - USER_UID=1000"
	echo "      - USER_GID=1000"
	echo "      - DATABASE_URL=/var/log/mock-gitea"
	echo "      - MGITEA__DATA=/var/lib/unique.txt"
	echo "      - DATABASE_URL=/var/lib/admin.db"
	echo "      - STARCHART__LOG=info"
	echo "      - STARCHART__SOURCE_CODE=https://github.com/forgeflux-org/starchart"
	echo "      - STARCHART__ALLOW_NEW_INDEX=true"
	echo "      - STARCHART__ADMIN_EMAIL=realaravinth@batsense.net"
	echo "      - STARCHART__SERVER__IP=0.0.0.0"
	echo "      - STARCHART__SERVER__PORT=$port"
	echo "      - STARCHART__SERVER__DOMAIN=localhost"
	echo "      - STARCHART__SERVER__PROXY_HAS_TLS=false"
	echo "      - STARCHART__SERVER__COOKIE_SECRET=7514316e58bfdb2eb2d71bf4af40827a"
	echo "      - STARCHART__DATABASE__POOL=5"
	echo "      - STARCHART__DATABASE__TYPE=sqlite"
	echo "      - STARCHART__CRAWLER__TTL=3600"
	echo "      - STARCHART__CRAWLER__WAIT_BEFORE_NEXT_API_CALL=2"
	echo "      - STARCHART__CRAWLER__CLIENT_TIMEOUT=60"
	echo "      - STARCHART__CRAWLER__ITEMS_PER_API_CALL=20"
	echo "      - STARCHART__INTRODUCER__WAIT=10"
	echo "      - STARCHART__INTRODUCER__PUBLIC_URL=http://localhost:$port"
	if [ $count -gt 0 ]
	then
		echo "      - STARCHART__INTRODUCER__NODES=$introducer"
	fi
	echo "      - STARCHART__REPOSITORY__ROOT=/tmp/starchart.forgeflux.org"
	echo "    restart: always"
	echo '    network_mode: "host"'
	echo "    ports:"
	echo "     - '$port:$port'"
	if [ $count -gt 0 ]
	then
	echo "    depends_on:"
	for depends_on_count in $(seq 0 $(expr $count - 1))
		do
			depends_on_container="$depends_on_count.starchart.test.forgeflux.org"
			echo "      - $depends_on_container"
		done
        introducer="$introducer,http://localhost:$port"
    else
        introducer="http://localhost:$port"
	fi

done
