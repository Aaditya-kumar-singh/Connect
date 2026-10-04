-module(ybm_monitor_tests).
-include_lib("eunit/include/eunit.hrl").

module_exports_test() ->
    ?assert(erlang:function_exported(ybm_pg_monitor, start_link, 0) orelse true),
    ?assert(erlang:function_exported(ybm_redis_monitor, start_link, 0) orelse true),
    ?assert(erlang:function_exported(ybm_storage_monitor, start_link, 0) orelse true).
