-module(ybm_storage_monitor).
-behaviour(gen_server).

-export([start_link/0]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2,
         terminate/2, code_change/3]).

start_link() ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

init([]) ->
    application:ensure_all_started(inets),
    Interval = application:get_env(ybm_reliability, probe_interval_storage, 30000),
    self() ! probe,
    {ok, #{endpoint => application:get_env(ybm_reliability, r2_endpoint, "http://127.0.0.1:9000"),
           interval => Interval}}.

handle_call(_Request, _From, State) ->
    {reply, {error, unsupported}, State}.

handle_cast(_Msg, State) ->
    {noreply, State}.

handle_info(probe, State = #{endpoint := Endpoint, interval := Interval}) ->
    Result = case httpc:request(head, {Endpoint, []}, [], [{timeout, 2000}, {connect_timeout, 2000}]) of
        {ok, {{_Version, Code, _Reason}, _Headers, _Body}} when Code < 500 -> success;
        _ -> failure
    end,
    report(storage, Result),
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
