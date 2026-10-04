-module(ybm_circuit_breaker_tests).
-include_lib("eunit/include/eunit.hrl").

circuit_transitions_test() ->
    {ok, Pid} = ybm_circuit_breaker:start_link(#{
        name => test_dependency,
        threshold => 2,
        cooldown_ms => 20,
        success_threshold => 1
    }),
    ybm_circuit_breaker:report_failure(test_dependency),
    ?assertEqual(closed, maps:get(status, ybm_circuit_breaker:get_state(test_dependency))),
    ybm_circuit_breaker:report_failure(test_dependency),
    ?assertEqual(open, maps:get(status, ybm_circuit_breaker:get_state(test_dependency))),
    timer:sleep(30),
    ?assertEqual(half_open, maps:get(status, ybm_circuit_breaker:get_state(test_dependency))),
    ybm_circuit_breaker:report_success(test_dependency),
    timer:sleep(5),
    ?assertEqual(closed, maps:get(status, ybm_circuit_breaker:get_state(test_dependency))),
    exit(Pid, shutdown).
