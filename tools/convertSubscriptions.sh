#!/bin/bash

tail subscriptions.db -n +2 | sed "s/\"//g" | sed "s/^/youtube,/" > subscriptions.csv
