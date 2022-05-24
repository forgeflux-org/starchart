#!/bin/bash

readonly username=starchart
USER_ID=${LOCAL_USER_ID}
echo "[*] Local user ID: $USER_ID"
echo "[*] Starting with UID : $USER_ID"
export HOME=/home/$username
#adduser --disabled-password --shell /bin/bash --home $HOME --uid $USER_ID user
#--uid
useradd --uid $USER_ID -b /home -m -s /bin/bash $username
su - $username 
starchart
