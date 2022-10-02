#!/usr/bin/env bash

set -e
shopt -s expand_aliases

if ! [ -x "$(command -v gsed)" ]; then
    alias sed_cmd="sed"
else
    alias sed_cmd="gsed"
fi

FILE=.env
if [[ -f "$FILE" ]]; then
    echo "$FILE exists already - exiting"
    exit 0
fi

if [[ -z $POSTGRES_DB || -z $POSTGRES_HOST || -z $POSTGRES_PASSWORD || -z $POSTGRES_USER ]]; then
    >&2 echo "Please define following env variables POSTGRES_DB, POSTGRES_USER, POSTGRES_HOST, POSTGRES_PASSWORD"
    exit 999
fi

cp .env-example $FILE

sed_cmd -i "s/\"postgres\"/\"${POSTGRES_USER}\"/g" $FILE
sed_cmd -i "s/\"password\"/\"${POSTGRES_PASSWORD}\"/g" $FILE
sed_cmd -i "s/postgres\:password/${POSTGRES_USER}\:${POSTGRES_PASSWORD}/g" $FILE
sed_cmd -i "s/localhost/${POSTGRES_HOST}/g" $FILE
sed_cmd -i "s/swu/${POSTGRES_DB}/g" $FILE
