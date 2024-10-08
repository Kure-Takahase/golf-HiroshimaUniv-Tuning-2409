# ベースステージ
FROM node:20.14.0 AS base

# 開発用ステージ
FROM base AS development

WORKDIR /usr/src/frontend

COPY package.json yarn.lock ./


# ビルドステージ
FROM hirouniv2409.azurecr.io/frontend:base AS build

WORKDIR /usr/src/frontend

COPY ./ ./

RUN --mount=type=cache,target=/usr/local/share/.cache/yarn/v6 \
    yarn install --frozen-lockfile

RUN yarn build


# 本番用ステージ
FROM hirouniv2409.azurecr.io/frontend:base AS production

WORKDIR /usr/src/frontend

COPY --from=build /usr/src/frontend/package.json ./
COPY --from=build /usr/src/frontend/yarn.lock ./
COPY --from=build /usr/src/frontend/.next ./.next
COPY --from=build /usr/src/frontend/public ./public
RUN --mount=type=cache,target=/usr/local/share/.cache/yarn/v6 \
    yarn install --production --frozen-lockfile

EXPOSE 3000
