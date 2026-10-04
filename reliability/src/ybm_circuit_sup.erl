-module(ybm_circuit_sup).
-behaviour(supervisor).

-export([start_link/0, init/1]).

start_link() ->
    supervisor:start_link({local, ?MODULE}, ?MODULE, []).

init([]) ->
    {ok, {{one_for_one, 10, 60}, [
        child(postgres, pg_opts()),
        child(redis, redis_opts()),
        child(storage, storage_opts())
    ]}}.

child(Name, Opts) ->
    {ybm_circuit_breaker:name(Name),
     {ybm_circuit_breaker, start_link, [maps:put(name, Name, Opts)]},
     permanent, 5000, worker, [ybm_circuit_breaker]}.

pg_opts() ->
    #{threshold => application:get_env(ybm_reliability, pg_failure_threshold, 5),
      cooldown_ms => application:get_env(ybm_reliability, pg_cooldown_ms, 30000),
      success_threshold => application:get_env(ybm_reliability, pg_success_threshold, 2)}.

redis_opts() ->
    #{threshold => application:get_env(ybm_reliability, redis_failure_threshold, 5),
      cooldown_ms => application:get_env(ybm_reliability, redis_cooldown_ms, 15000),
      success_threshold => application:get_env(ybm_reliability, redis_success_threshold, 2)}.

storage_opts() ->
    #{threshold => application:get_env(ybm_reliability, storage_failure_threshold, 3),
      cooldown_ms => application:get_env(ybm_reliability, storage_cooldown_ms, 30000),
      success_threshold => application:get_env(ybm_reliability, storage_success_threshold, 1)}.
