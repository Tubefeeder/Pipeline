#!/bin/sh

sed 's/"url"/\n&/g' $1 | sed '1d' | awk -F "," '//{print $1}' | sed 's/"//g' | awk -F "/" '// {printf "\"%s\"\n", $(NF)}' | sed '1i key str "channel_id"'
