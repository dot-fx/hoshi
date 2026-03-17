FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    chromium \
    ca-certificates \
    fonts-liberation \
    curl \
    jq \
    --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

ENV CHROME_BIN=/usr/bin/chromium

RUN LATEST_URL=$(curl -s https://api.github.com/repos/dot-fx/hoshi/releases/latest \
    | jq -r '.assets[] | select(.name | startswith("hoshi-server-linux")) | .browser_download_url') \
    && echo "Descargando binario desde: $LATEST_URL" \
    && curl -L "$LATEST_URL" -o /app/hoshi-server \
    && chmod +x /app/hoshi-server

EXPOSE 10090

CMD ["/app/hoshi-server"]