-module(ybm_redis_monitor).
-behaviour(gen_server).

-export([start_link/0]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2,
         terminate/2, code_change/3]).

start_link() ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

init([]) ->
    Interval = application:get_env(ybm_reliability, probe_interval_redis, 10000),
    self() ! probe,
    {ok, #{host => application:get_env(ybm_reliability, redis_host, "127.0.0.1"),
           port => application:get_env(ybm_reliability, redis_port, 6379),
           password => application:get_env(ybm_reliability, redis_password, ""),
           interval => Interval}}.

handle_call(_Request, _From, State) ->
    {reply, {error, unsupported}, State}.

handle_cast(_Msg, State) ->
    {noreply, State}.

handle_info(probe, State = #{host := Host, port := Port, password := Password, interval := Interval}) ->
    Result = case eredis:start_link(Host, Port, 0, Password, no_reconnect, 2000) of
        {ok, Client} ->
            Reply = eredis:q(Client, ["PING"]),
            eredis_client:stop(Client),
            case Reply of
                {ok, <<"PONG">>} -> success;
                _ -> failure
            end;
        {error, _} ->
            failure
    end,
    report(redis, Result),
    erlang:send_after(Interval, self(), probe),
    {noreply, State};
handle_info(_Info, State) ->
    {noreply, State}.

report(Name, success) -> ybm_circuit_breaker:report_success(Name);
report(Name, failure) -> ybm_circuit_breaker:report_failure(Name).

terminate(_Reason, _State) ->
    ok.

code_change(_OldVsn, State, _Extra) ->
    {ok, State}.
