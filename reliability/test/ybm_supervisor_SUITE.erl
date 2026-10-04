-module(ybm_supervisor_SUITE).
-compile(export_all).
-include_lib("common_test/include/ct.hrl").

all() ->
    [application_lifecycle, sibling_isolation].

init_per_suite(Config) ->
    application:ensure_all_started(ybm_reliability),
    Config.

end_per_suite(_Config) ->
    application:stop(ybm_reliability),
    ok.

application_lifecycle(_Config) ->
    ?assertMatch({ok, _}, application:ensure_all_started(ybm_reliability)),
    ok.

sibling_isolation(_Config) ->
    {ok, Pid} = ybm_circuit_breaker:start_link(#{
        name => ct_dependency,
        threshold => 2,
        cooldown_ms => 1000,
        success_threshold => 1
    }),
    exit(Pid, kill),
    timer:sleep(20),
    ?assert(is_process_alive(whereis(ct_dependency))).
