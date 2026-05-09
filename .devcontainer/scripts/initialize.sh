#!/bin/bash

cp .env.dev.dist .env

set -a && . ./.env && set +a
