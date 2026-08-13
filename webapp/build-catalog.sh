#!/bin/sh
set -eu
commit="da90466031b0372c896588b85be6016c617e205b"
curl --fail --silent --show-error --location "https://raw.githubusercontent.com/mattiaverga/OpenNGC/${commit}/database_files/NGC.csv" -o NGC.csv
curl --fail --silent --show-error --location "https://raw.githubusercontent.com/mattiaverga/OpenNGC/${commit}/database_files/addendum.csv" -o addendum.csv
python3 build_catalog.py NGC.csv addendum.csv catalog.tsv
