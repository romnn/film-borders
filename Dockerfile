FROM rust:1.61 as wasmbuild

WORKDIR /app
LABEL MAINTAINER="roman <contact@romnn.com>"

RUN apt-get update && apt-get install -y python3 python3-pip nodejs npm
RUN cargo install wasm-pack
RUN pip3 install invoke
RUN npm install --global yarn

ADD ./ /app
RUN invoke pack

ARG version=0.0.1
ARG publicURL
ENV PUBLIC_URL=$publicURL

RUN echo "Using version ${version}"
RUN echo "Using public URL ${PUBLIC_URL}"
RUN cd /app/www \
  && npm version --no-git-tag-version ${version} \
  && yarn install \
  && yarn build

FROM nginx:alpine
EXPOSE 80
RUN mkdir /app
COPY --from=wasmbuild /app/www/build /serve
COPY --from=wasmbuild /app/nginx.conf /etc/nginx/nginx.conf
