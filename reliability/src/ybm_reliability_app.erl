-module(ybm_reliability_app).
-behaviour(application).

-export([start/2, stop/1]).

start(_StartType, _StartArgs) ->
    configure_from_env(),
    ybm_reliability_sup:start_link().

configure_from_env() ->
    set_env(redis_host, "REDIS_HOST", "127.0.0.1"),
    set_env(redis_port, "REDIS_PORT", 6379, integer),
    set_env(redis_password, "REDIS_PASSWORD", ""),
    set_env(pg_host, "PG_HOST", "127.0.0.1"),
    set_env(pg_port, "PG_PORT", 5432, integer),
    set_env(r2_endpoint, "R2_ENDPOINT", "http://127.0.0.1:9000"),
    set_env(health_port, "HEALTH_PORT", 8081, integer),
    set_env(probe_interval_pg, "PROBE_INTERVAL_PG", 15000, integer),
    set_env(probe_interval_redis, "PROBE_INTERVAL_REDIS", 10000, integer),
    set_env(probe_interval_storage, "PROBE_INTERVAL_STORAGE", 30000, integer),
    ok.

set_env(Key, Name, Default) ->
    case os:getenv(Name) of
        false -> ok;
        Value -> application:set_env(ybm_reliability, Key, Value)
    end,
    case os:getenv(Name) of
        false -> application:set_env(ybm_reliability, Key, Default);
        _ -> ok
    end.

set_env(Key, Name, Default, integer) ->
    Value = case os:getenv(Name) of
        false -> Default;
        Raw -> list_to_integer(Raw)
    end,
    application:set_env(ybm_reliability, Key, Value).

stop(_State) ->
    ok.
