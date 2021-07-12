FROM rust:1.53 as wasmbuild

WORKDIR /app
ADD ./ /app

ARG version=0.0.1
LABEL MAINTAINER="roman <contact@romnn.com>"

RUN apt-get update && apt-get install -y python3 python3-pip nodejs npm
RUN cargo install wasm-pack
RUN pip3 install invoke
RUN npm install --global yarn
RUN invoke pack
RUN echo "Using version ${version}"
RUN cd /app/www && npm version --no-git-tag-version ${version} && yarn install && yarn build

FROM nginx:latest
EXPOSE 80
RUN mkdir /app
COPY --from=wasmbuild /app/www/build /serve
COPY --from=wasmbuild /app/nginx.conf /etc/nginx/nginx.conf
