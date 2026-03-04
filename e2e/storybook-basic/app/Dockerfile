FROM node:20-alpine AS builder
WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm install
COPY . .
RUN npx storybook build -o storybook-static

FROM node:20-alpine
RUN npm install -g http-server@14
WORKDIR /app
COPY --from=builder /app/storybook-static ./storybook-static
EXPOSE 8080
HEALTHCHECK --interval=2s --timeout=3s --start-period=15s --retries=15 \
  CMD wget -q --spider http://127.0.0.1:8080/ || exit 1
CMD ["http-server", "storybook-static", "-p", "8080", "--silent", "-c-1"]
