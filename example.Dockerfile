
FROM sphinxdoc/sphinx:3.5.3 AS html

FROM node:lts-alpine3.11 AS editor

FROM mverleg/pastacode-base:latest AS wasm

FROM nginx:1.9.14-alpine AS host

