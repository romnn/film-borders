FROM rust:1.61 as build

ARG version=0.0.1
ARG publicURL
ENV PUBLIC_URL=$publicURL

RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash -
RUN apt-get update && apt-get install -y python3 python3-pip nodejs

RUN cargo install wasm-pack
RUN pip3 install invoke
RUN npm install --global yarn

WORKDIR /build
ADD ./src /build/src
ADD ./Cargo.* /build/
ADD ./tasks.py /build/
ADD ./www /build/www
RUN ls -lia
RUN invoke pack

RUN echo "Using version ${version}"
RUN echo "Using public URL ${PUBLIC_URL}"
RUN cd /build/www \
  && npm version --no-git-tag-version ${version} \
  && yarn install \
  && yarn build

FROM nginx:alpine
EXPOSE 80
ADD ./nginx.conf /etc/nginx/nginx.conf
COPY --from=build /build/www/build /serve
