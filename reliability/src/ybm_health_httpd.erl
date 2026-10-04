-module(ybm_health_httpd).

-include_lib("inets/include/httpd.hrl").

-export([do/1]).

do(#mod{method = "GET", request_uri = "/health"}) ->
    {break, [{response, {200, [{"content-type", "application/json"},
                               {"cache-control", "no-store"}],
                         ybm_health_aggregator:health_json()}}]};
do(#mod{method = "HEAD", request_uri = "/health"}) ->
    {break, [{response, {200, [{"content-type", "application/json"},
                               {"cache-control", "no-store"}],
                         nobody}}]};
do(_ModData) ->
    {proceed, []}.
