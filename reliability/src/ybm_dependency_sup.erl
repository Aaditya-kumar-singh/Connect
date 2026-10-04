-module(ybm_dependency_sup).
-behaviour(supervisor).

-export([start_link/0, init/1]).

start_link() ->
    supervisor:start_link({local, ?MODULE}, ?MODULE, []).

init([]) ->
    {ok, {{one_for_one, 10, 60}, [
        {ybm_pg_monitor, {ybm_pg_monitor, start_link, []},
         permanent, 5000, worker, [ybm_pg_monitor]},
        {ybm_redis_monitor, {ybm_redis_monitor, start_link, []},
         permanent, 5000, worker, [ybm_redis_monitor]},
        {ybm_storage_monitor, {ybm_storage_monitor, start_link, []},
         permanent, 5000, worker, [ybm_storage_monitor]}
    ]}}.
