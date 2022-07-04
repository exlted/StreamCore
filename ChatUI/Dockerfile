# Install a dartchat as a docker container.
# Dartchat will be exposed via HTTP on port 3000.
#
# This file is hosted on github. Therefore you can start it in docker like this:
# > docker build -t dartchat github.com/nkratzke/dartchat
# > docker run -p 8080:3000 -d dartchat

FROM dart:stable AS build

WORKDIR /app
COPY pubspec.* ./
RUN dart pub get

COPY . .

RUN dart pub get --offline
RUN dart compile exe bin/server.dart -o bin/server

FROM scratch
COPY --from=build /runtime/ /
COPY --from=build /app/bin/server /app/bin/
COPY public/*.* /public/

# Expose port 3000.
EXPOSE 8080

# Change this to your starting dart.
CMD ["/app/bin/server"]