-module(ybm_pg_monitor).
-behaviour(gen_server).

-export([start_link/0]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2,
         terminate/2, code_change/3]).

start_link() ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

init([]) ->
    Interval = application:get_env(ybm_reliability, probe_interval_pg, 15000),
    self() ! probe,
    {ok, #{host => application:get_env(ybm_reliability, pg_host, "127.0.0.1"),
           port => application:get_env(ybm_reliability, pg_port, 5432),
           interval => Interval}}.

handle_call(_Request, _From, State) ->
    {reply, {error, unsupported}, State}.

handle_cast(_Msg, State) ->
    {noreply, State}.

handle_info(probe, State = #{host := Host, port := Port, interval := Interval}) ->
    Result = case gen_tcp:connect(Host, Port, [binary, {active, false}], 2000) of
        {ok, Socket} ->
            gen_tcp:close(Socket),
            success;
        {error, _} ->
            failure
    end,
    report(postgres, Result),
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
