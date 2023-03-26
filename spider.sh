#!/bin/bash

set -euo pipefail

# $1: starchart instance URL without the trailing slash
# $2: gitea instace URL
spider() {
	echo "[*] Ordering $1 to crawl $2"
	curl "$1/verify" -vv -L \
		-X POST \
		-H 'Content-Type: application/x-www-form-urlencoded' \
		 --data-urlencode "hostname=$2"
 }

starchart="https://starchart.forgeflux.org"
starchart="http://localhost:12001"
gitea_url="http://localhost:11000"


spider $starchart $gitea_url

#for count in {0..100}
#do
#	gitea=$(expr $count + 11000)
#	top=0
#	while true
#	do
#		i=$(expr 110 + $top)
#		i=$(expr $i \* 10)
#		if [ $(expr $gitea % $i) -eq $gitea ]
#		then
#			break
#		else
#			top="$(expr $top + 1)0"
#		fi
#	done
#	starchart="http://localhost:1200$top"
#	echo $starchart
#done
