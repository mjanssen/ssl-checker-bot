FROM alpine:3.18

ARG TARGETARCH

RUN apk add --no-cache dumb-init

COPY --chmod=755 bot_$TARGETARCH /usr/local/bin/bot
COPY --chmod=755 cron_$TARGETARCH /usr/local/bin/cron

RUN echo -n `date '+v%Y.%m.%d.%H.%M'` > /etc/ssl_check_bot_build
