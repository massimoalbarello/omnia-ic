#!/bin/sh
# commands executed when building frontends
cd src/omnia_user_frontend && npm run build
cd ../..
cd src/omnia_gateway_frontend && npm run build