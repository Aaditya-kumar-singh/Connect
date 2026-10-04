-module(ybm_reliability_sup).
-behaviour(supervisor).

-export([start_link/0, init/1]).

start_link() ->
    supervisor:start_link({local, ?MODULE}, ?MODULE, []).

init([]) ->
    {ok, {{one_for_one, 10, 60}, [
        {ybm_redis_pubsub, {ybm_redis_pubsub, start_link, []},
         permanent, 5000, worker, [ybm_redis_pubsub]},
        {ybm_circuit_sup, {ybm_circuit_sup, start_link, []},
         permanent, 5000, supervisor, [ybm_circuit_sup]},
        {ybm_dependency_sup, {ybm_dependency_sup, start_link, []},
         permanent, 5000, supervisor, [ybm_dependency_sup]},
        {ybm_health_sup, {ybm_health_sup, start_link, []},
         permanent, 5000, supervisor, [ybm_health_sup]},
        {ybm_recovery_sup, {ybm_recovery_sup, start_link, []},
         permanent, 5000, supervisor, [ybm_recovery_sup]}
    ]}}.
