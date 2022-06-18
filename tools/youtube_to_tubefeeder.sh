#!/bin/sh

cat $1 | sed '1d' | awk -F "," '//{print "youtube,"$1}'
