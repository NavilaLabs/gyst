#!/bin/bash

cp .env.dev.dist .env.dev
cp .env.test.dist .env.test

set -a && . ./.env.dev && set +a
