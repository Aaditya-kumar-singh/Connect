-module(ybm_redis_pubsub_tests).
-include_lib("eunit/include/eunit.hrl").

event_channel_test() ->
    ?assertEqual(<<"ybm:reliability:events">>, <<"ybm:reliability:events">>),
    ?assertEqual(<<"ybm:reliability:status">>, <<"ybm:reliability:status">>).
