FROM node:21-alpine3.18 as build-stage
WORKDIR /app
COPY package*.json .npmrc ./ 
RUN npm install

FROM build-stage as build-stage-prod
COPY . .
RUN npm run build:production

FROM build-stage as build-stage-dev
COPY . .
RUN npm run build:develop

FROM nginx:stable-alpine as prod-image
WORKDIR /app
COPY --from=build-stage-prod /app/dist /usr/share/nginx/html
COPY ./default.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]

