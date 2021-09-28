#!/bin/bash

if [ -f subscriptions.db ]; then
    tail subscriptions.db -n +2 | sed "s/\"//g" | sed "s/^/youtube,/" > subscriptions.csv
else
    echo "Subscriptions file does not exist"
fi

if [ -f filters.db ]; then
    tail filters.db -n +2 | sed "s/\"//g" | sed "s/^/youtube,/" > filters.csv
else
    echo "Filters file does not exist"
fi

if [ -f watch_later.db ]; then
    tail watch_later.db -n +2 | sed "s/\"//g" | sed "s/https:\/\/www.youtube.com\/channel\///g" | sed "s/+00:00//g" | sed "s/^/youtube,/" > playlist_watch_later.csv
else
    echo "Watch later file does not exist"
fi
