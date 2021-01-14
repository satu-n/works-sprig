FROM alpine AS elm
RUN wget -O - https://github.com/elm/compiler/releases/download/0.19.1/binary-for-linux-64-bit.gz \
    | gunzip -c >/usr/local/bin/elm
RUN chmod +x /usr/local/bin/elm

FROM elm AS build
WORKDIR /usr/local/src
COPY . .
RUN elm make src/Main.elm --optimize
RUN sed -i 's#<style>body { padding: 0; margin: 0; }</style>#<link rel="stylesheet" type="text/css" href="css/style.css">#' index.html

FROM nginx:alpine
COPY --from=build /usr/local/src/index.html /usr/local/src/css /usr/share/nginx/html/
