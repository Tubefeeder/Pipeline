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
