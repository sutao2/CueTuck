FROM node:22-bookworm-slim@sha256:83f487e0a63425e5b4d146fb5e5be574bcbe1b7b843d3ebafdd95eaf7767a7e5 AS build
WORKDIR /build/admin-web
COPY admin-web/package.json admin-web/package-lock.json ./
RUN npm ci
COPY admin-web/ ./
COPY shared/ /build/shared/
COPY desktop/src/styles/tokens.css /build/desktop/src/styles/tokens.css
COPY desktop/src/components/SearchableSelect.vue /build/desktop/src/components/SearchableSelect.vue
COPY desktop/src/components/AppIcon.vue /build/desktop/src/components/AppIcon.vue
COPY desktop/src/assets/app-icon.png /build/desktop/src/assets/app-icon.png
ARG VITE_API_BASE=https://prompt-admin.likh.cn
ENV VITE_API_BASE=$VITE_API_BASE
RUN npm run build
FROM nginx:1.28-alpine@sha256:a8b39bd9cf0f83869a2162827a0caf6137ddf759d50a171451b335cecc87d236
COPY deploy/production/admin.nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /build/admin-web/dist /usr/share/nginx/html
