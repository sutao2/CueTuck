#!/bin/sh
set -eu
cd /opt/promptark/source/deploy/production
. ./images.env
docker run --rm -v /opt/promptark/letsencrypt:/etc/letsencrypt -v /www/wwwroot/promptark-acme:/var/www/acme "$CERTBOT_IMAGE" renew --quiet
/www/server/nginx/sbin/nginx -t
/www/server/nginx/sbin/nginx -s reload
