FROM node:24-alpine AS builder
WORKDIR /app
RUN corepack enable
COPY notification-frontend/package.json notification-frontend/pnpm-lock.yaml notification-frontend/pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY notification-frontend/ ./
RUN pnpm build

FROM nginxinc/nginx-unprivileged:1.27-alpine
COPY notification-frontend/nginx.conf.template /etc/nginx/templates/default.conf.template
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=3s --retries=3 CMD wget -q -O /dev/null http://127.0.0.1:8080/healthz || exit 1
