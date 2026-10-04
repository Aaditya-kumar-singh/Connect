FROM erlang:27.3.4.17-alpine AS build

RUN apk add --no-cache curl git
RUN curl -fsSL https://github.com/erlang/rebar3/releases/download/3.27.0/rebar3 -o /usr/local/bin/rebar3     && chmod +x /usr/local/bin/rebar3

WORKDIR /app
COPY rebar.config .
COPY src ./src
COPY test ./test
COPY README.md .

RUN rebar3 compile
RUN rebar3 as prod release

FROM erlang:27.3.4.17-alpine
WORKDIR /app
COPY --from=build /app/_build/prod/rel/ybm_reliability ./

ENV REDIS_HOST=redis     REDIS_PORT=6379     REDIS_PASSWORD=     PG_HOST=postgres     PG_PORT=5432     R2_ENDPOINT=http://minio:9000     HEALTH_PORT=8081

EXPOSE 8081
CMD ["bin/ybm_reliability", "foreground"]
