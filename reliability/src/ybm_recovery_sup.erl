-module(ybm_recovery_sup).
-behaviour(supervisor).

-export([start_link/0, init/1]).

start_link() ->
    supervisor:start_link({local, ?MODULE}, ?MODULE, []).

init([]) ->
    {ok, {{one_for_one, 5, 60}, [
        {ybm_recovery_coordinator, {ybm_recovery_coordinator, start_link, []},
         permanent, 5000, worker, [ybm_recovery_coordinator]}
    ]}}.
