-module(ybm_health_sup).
-behaviour(supervisor).

-export([start_link/0, init/1]).

start_link() ->
    supervisor:start_link({local, ?MODULE}, ?MODULE, []).

init([]) ->
    {ok, {{one_for_one, 5, 60}, [
        {ybm_health_aggregator, {ybm_health_aggregator, start_link, []},
         permanent, 5000, worker, [ybm_health_aggregator]}
    ]}}.
